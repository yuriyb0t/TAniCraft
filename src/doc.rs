//! Document model: layers, frames, raster bitmaps and the .tcraft file format.

use std::io::{BufWriter, Cursor, Read, Write};
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub const MAGIC: &[u8; 8] = b"TACRAFT1";
pub const FILE_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Bitmap
// ---------------------------------------------------------------------------

/// Premultiplied RGBA8 raster. Cloning is cheap (shared buffer, copy-on-write).
#[derive(Clone, Debug)]
pub struct Bitmap {
    pub w: usize,
    pub h: usize,
    data: Arc<Vec<u8>>,
    pub rev: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl DirtyRect {
    pub fn union(self, o: DirtyRect) -> DirtyRect {
        let x0 = self.x.min(o.x);
        let y0 = self.y.min(o.y);
        let x1 = (self.x + self.w).max(o.x + o.w);
        let y1 = (self.y + self.h).max(o.y + o.h);
        DirtyRect { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
    }
}

impl Bitmap {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, data: Arc::new(vec![0; w * h * 4]), rev: 1 }
    }

    pub fn from_rgba_straight(w: usize, h: usize, src: &[u8]) -> Self {
        let mut bmp = Self::new(w, h);
        let data = Arc::make_mut(&mut bmp.data);
        for i in 0..(w * h) {
            let a = src[i * 4 + 3] as u32;
            for c in 0..3 {
                data[i * 4 + c] = ((src[i * 4 + c] as u32 * a) / 255) as u8;
            }
            data[i * 4 + 3] = a as u8;
        }
        bmp.rev = 1;
        bmp
    }

    pub fn pixels(&self) -> &[u8] {
        &self.data
    }

    pub fn is_blank(&self) -> bool {
        self.data.chunks_exact(4).all(|p| p[3] == 0)
    }

    fn data_mut(&mut self) -> &mut Vec<u8> {
        self.rev = self.rev.wrapping_add(1);
        Arc::make_mut(&mut self.data)
    }

    pub fn clear(&mut self) {
        let d = self.data_mut();
        d.iter_mut().for_each(|v| *v = 0);
    }

    pub fn fill(&mut self, rgba: [u8; 4]) {
        let d = self.data_mut();
        for p in d.chunks_exact_mut(4) {
            p.copy_from_slice(&rgba);
        }
    }

    /// Stamp a soft round brush dab. Colours are premultiplied RGBA in 0..=1.
    /// `alpha` is the paint opacity (0..=1), `erase` multiplies the destination
    /// down to transparent instead of painting.
    pub fn stamp(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        hardness: f32,
        color: [f32; 3],
        alpha: f32,
        erase: bool,
    ) -> Option<DirtyRect> {
        if radius <= 0.01 || alpha <= 0.0 {
            return None;
        }
        let x0 = (cx - radius - 1.0).floor().max(0.0) as i64;
        let y0 = (cy - radius - 1.0).floor().max(0.0) as i64;
        let x1 = (cx + radius + 1.0).ceil().min(self.w as f32) as i64;
        let y1 = (cy + radius + 1.0).ceil().min(self.h as f32) as i64;
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        let hard = hardness.clamp(0.0, 1.0);
        let inner = radius * hard;
        let band = (radius - inner).max(0.0001);
        let r2 = radius * radius;
        let bw = self.w;
        let data = self.data_mut();
        for y in y0..y1 {
            let dy = y as f32 + 0.5 - cy;
            for x in x0..x1 {
                let dx = x as f32 + 0.5 - cx;
                let d2 = dx * dx + dy * dy;
                if d2 > r2 {
                    continue;
                }
                let d = d2.sqrt();
                let mut cov = if d <= inner {
                    1.0
                } else {
                    let t = 1.0 - (d - inner) / band;
                    // smoothstep for a soft, even falloff
                    t * t * (3.0 - 2.0 * t)
                };
                if cov <= 0.0 {
                    continue;
                }
                cov *= alpha;
                let idx = ((y as usize) * bw + x as usize) * 4;
                if erase {
                    let inv = 1.0 - cov;
                    for c in 0..4 {
                        data[idx + c] = (data[idx + c] as f32 * inv) as u8;
                    }
                } else {
                    let inv = 1.0 - cov;
                    for c in 0..3 {
                        let src = color[c] * cov;
                        data[idx + c] = (data[idx + c] as f32 * inv + src * 255.0) as u8;
                    }
                    data[idx + 3] = (data[idx + 3] as f32 * inv + cov * 255.0) as u8;
                }
            }
        }
        Some(DirtyRect { x: x0 as u32, y: y0 as u32, w: (x1 - x0) as u32, h: (y1 - y0) as u32 })
    }

    /// Composite `src` over `self` at the given offset (premultiplied).
    pub fn draw_bitmap(&mut self, src: &Bitmap, ox: i32, oy: i32, opacity: f32) {
        if opacity <= 0.0 {
            return;
        }
        let (dw, dh) = (self.w, self.h);
        let data = self.data_mut();
        for sy in 0..src.h {
            let ty = sy as i32 + oy;
            if ty < 0 || ty >= dh as i32 {
                continue;
            }
            for sx in 0..src.w {
                let tx = sx as i32 + ox;
                if tx < 0 || tx >= dw as i32 {
                    continue;
                }
                let si = (sy * src.w + sx) * 4;
                let sa = src.data[si + 3] as f32 / 255.0 * opacity;
                if sa <= 0.0 {
                    continue;
                }
                let di = (ty as usize * dw + tx as usize) * 4;
                let inv = 1.0 - sa;
                for c in 0..4 {
                    let s = src.data[si + c] as f32 * opacity;
                    data[di + c] = (data[di + c] as f32 * inv + s).min(255.0) as u8;
                }
            }
        }
    }

    pub fn to_color_image(&self) -> egui::ColorImage {
        egui::ColorImage::from_rgba_premultiplied([self.w, self.h], &self.data)
    }

    pub fn region_image(&self, r: DirtyRect) -> egui::ColorImage {
        let mut out = Vec::with_capacity((r.w * r.h * 4) as usize);
        for y in r.y..r.y + r.h {
            let start = ((y as usize) * self.w + r.x as usize) * 4;
            out.extend_from_slice(&self.data[start..start + r.w as usize * 4]);
        }
        egui::ColorImage::from_rgba_premultiplied([r.w as usize, r.h as usize], &out)
    }

    pub fn to_rgba_straight(&self) -> Vec<u8> {
        let mut out = self.data.as_ref().clone();
        for p in out.chunks_exact_mut(4) {
            let a = p[3] as u32;
            if a == 0 {
                p[0] = 0;
                p[1] = 0;
                p[2] = 0;
            } else if a < 255 {
                for c in 0..3 {
                    p[c] = ((p[c] as u32 * 255) / a).min(255) as u8;
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Frames & layers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FrameKind {
    /// No content and nothing held from the left.
    Empty,
    /// Continues the previous keyframe.
    Hold,
    /// Owns a bitmap (possibly blank).
    Key,
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub kind: FrameKind,
    pub bitmap: Option<Bitmap>,
}

impl Frame {
    pub fn empty() -> Self {
        Self { kind: FrameKind::Empty, bitmap: None }
    }
    pub fn hold() -> Self {
        Self { kind: FrameKind::Hold, bitmap: None }
    }
    pub fn key(w: usize, h: usize) -> Self {
        Self { kind: FrameKind::Key, bitmap: Some(Bitmap::new(w, h)) }
    }
    pub fn key_from(bitmap: Bitmap) -> Self {
        Self { kind: FrameKind::Key, bitmap: Some(bitmap) }
    }
    pub fn key_bitmap(&self) -> Option<&Bitmap> {
        if self.kind == FrameKind::Key {
            self.bitmap.as_ref()
        } else {
            None
        }
    }
    pub fn key_bitmap_mut(&mut self) -> Option<&mut Bitmap> {
        if self.kind == FrameKind::Key {
            self.bitmap.as_mut()
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub uid: u64,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub frames: Vec<Frame>,
}

impl Layer {
    pub fn new(uid: u64, name: impl Into<String>, w: usize, h: usize) -> Self {
        Self {
            uid,
            name: name.into(),
            visible: true,
            locked: false,
            frames: vec![Frame::key(w, h)],
        }
    }

    /// Index of the keyframe that provides the drawing for `frame`.
    pub fn effective_key(&self, frame: usize) -> Option<usize> {
        if frame >= self.frames.len() {
            return None;
        }
        for i in (0..=frame).rev() {
            match self.frames[i].kind {
                FrameKind::Key => return Some(i),
                FrameKind::Empty => return None,
                FrameKind::Hold => {}
            }
        }
        None
    }

    pub fn bitmap_at(&self, frame: usize) -> Option<&Bitmap> {
        let k = self.effective_key(frame)?;
        self.frames[k].key_bitmap()
    }

    pub fn bitmap_at_mut(&mut self, frame: usize) -> Option<&mut Bitmap> {
        let k = self.effective_key(frame)?;
        self.frames[k].key_bitmap_mut()
    }

    pub fn kind_at(&self, frame: usize) -> FrameKind {
        if frame >= self.frames.len() {
            FrameKind::Empty
        } else {
            self.frames[frame].kind
        }
    }

    pub fn keyframes(&self) -> impl Iterator<Item = usize> + '_ {
        self.frames
            .iter()
            .enumerate()
            .filter(|(_, f)| f.kind == FrameKind::Key)
            .map(|(i, _)| i)
    }
}

// ---------------------------------------------------------------------------
// Document
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Document {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    /// Straight (non-premultiplied) document background colour.
    pub bg: [u8; 4],
    /// Bottom-most layer first.
    pub layers: Vec<Layer>,
    next_uid: u64,
}

impl Document {
    pub fn new(name: impl Into<String>, width: u32, height: u32, fps: f32, bg: [u8; 4]) -> Self {
        let mut doc = Self {
            name: name.into(),
            width: width.max(1),
            height: height.max(1),
            fps: fps.clamp(1.0, 120.0),
            bg,
            layers: Vec::new(),
            next_uid: 1,
        };
        let uid = doc.take_uid();
        doc.layers.push(Layer::new(uid, "Layer 1", doc.width as usize, doc.height as usize));
        doc
    }

    fn take_uid(&mut self) -> u64 {
        let uid = self.next_uid;
        self.next_uid += 1;
        uid
    }

    pub fn frame_count(&self) -> usize {
        self.layers.iter().map(|l| l.frames.len()).max().unwrap_or(1).max(1)
    }

    pub fn layer_index(&self, uid: u64) -> Option<usize> {
        self.layers.iter().position(|l| l.uid == uid)
    }

    pub fn add_layer(&mut self, at_top: bool) -> usize {
        let uid = self.take_uid();
        let name = format!("Layer {}", self.layers.len() + 1);
        let mut layer = Layer::new(uid, name, self.width as usize, self.height as usize);
        // Match the document length so new layers do not cut the timeline short.
        let target = self.frame_count();
        while layer.frames.len() < target {
            layer.frames.push(Frame::hold());
        }
        if at_top || self.layers.is_empty() {
            self.layers.push(layer);
            self.layers.len() - 1
        } else {
            self.layers.insert(0, layer);
            0
        }
    }

    pub fn remove_layer(&mut self, idx: usize) -> Option<Layer> {
        if self.layers.len() <= 1 || idx >= self.layers.len() {
            return None;
        }
        Some(self.layers.remove(idx))
    }

    pub fn move_layer(&mut self, from: usize, to: usize) {
        if from >= self.layers.len() || to >= self.layers.len() || from == to {
            return;
        }
        let layer = self.layers.remove(from);
        self.layers.insert(to, layer);
    }

    /// Ensures `frame` is an editable keyframe on `layer_idx`.
    /// Holds are converted to keyframes that keep the inherited drawing, empty
    /// frames get a fresh blank bitmap.
    pub fn ensure_keyframe(&mut self, layer_idx: usize, frame: usize) -> bool {
        let (w, h) = (self.width as usize, self.height as usize);
        let Some(layer) = self.layers.get_mut(layer_idx) else {
            return false;
        };
        while layer.frames.len() <= frame {
            layer.frames.push(Frame::hold());
        }
        match layer.frames[frame].kind {
            FrameKind::Key => true,
            FrameKind::Hold => {
                let inherited = layer
                    .effective_key(frame.saturating_sub(1))
                    .and_then(|k| layer.frames[k].bitmap.clone());
                match inherited {
                    Some(bmp) => layer.frames[frame] = Frame::key_from(bmp),
                    None => layer.frames[frame] = Frame::key(w, h),
                }
                true
            }
            FrameKind::Empty => {
                layer.frames[frame] = Frame::key(w, h);
                true
            }
        }
    }

    /// Insert Keyframe (F6): duplicates the currently visible drawing.
    pub fn insert_keyframe(&mut self, layer_idx: usize, frame: usize, blank: bool) {
        let (w, h) = (self.width as usize, self.height as usize);
        let Some(layer) = self.layers.get_mut(layer_idx) else {
            return;
        };
        while layer.frames.len() < frame {
            layer.frames.push(Frame::hold());
        }
        let inherited = if blank {
            None
        } else {
            layer
                .effective_key(frame.saturating_sub(1))
                .and_then(|k| layer.frames[k].bitmap.clone())
        };
        let new = match inherited {
            Some(bmp) => Frame::key_from(bmp),
            None => Frame::key(w, h),
        };
        if frame < layer.frames.len() {
            layer.frames[frame] = new;
        } else {
            layer.frames.push(new);
        }
    }

    /// Insert or replace a keyframe with the supplied drawing.
    pub fn paste_keyframe(&mut self, layer_idx: usize, frame: usize, bitmap: Bitmap) -> bool {
        if bitmap.w != self.width as usize || bitmap.h != self.height as usize {
            return false;
        }
        let Some(layer) = self.layers.get_mut(layer_idx) else {
            return false;
        };
        while layer.frames.len() < frame {
            layer.frames.push(Frame::hold());
        }
        let key = Frame::key_from(bitmap);
        if frame < layer.frames.len() {
            layer.frames[frame] = key;
        } else {
            layer.frames.push(key);
        }
        true
    }

    /// Insert Frame (F5): extends the span that contains `frame`.
    pub fn insert_frame(&mut self, layer_idx: usize, frame: usize) {
        let Some(layer) = self.layers.get_mut(layer_idx) else {
            return;
        };
        if frame >= layer.frames.len() {
            while layer.frames.len() <= frame {
                layer.frames.push(Frame::hold());
            }
            return;
        }
        // Find the end of the current span and push the hold right before the
        // next keyframe so the visible drawing does not change.
        let mut at = frame + 1;
        while at < layer.frames.len() && layer.frames[at].kind == FrameKind::Hold {
            at += 1;
        }
        layer.frames.insert(at, Frame::hold());
    }

    /// Remove Frame (Shift+F5).
    pub fn remove_frame(&mut self, layer_idx: usize, frame: usize) {
        let Some(layer) = self.layers.get_mut(layer_idx) else {
            return;
        };
        if layer.frames.len() > 1 && frame < layer.frames.len() {
            layer.frames.remove(frame);
        }
    }

    /// Clear Keyframe (Shift+F6): turns the frame into a hold (or empty at the
    /// start of a span).
    pub fn clear_keyframe(&mut self, layer_idx: usize, frame: usize) {
        let Some(layer) = self.layers.get_mut(layer_idx) else {
            return;
        };
        if frame >= layer.frames.len() {
            return;
        }
        let preceding_has_content = layer
            .effective_key(frame.saturating_sub(1))
            .is_some()
            && frame > 0;
        if preceding_has_content {
            layer.frames[frame] = Frame::hold();
        } else {
            layer.frames[frame] = Frame::empty();
        }
    }

    /// Resize the canvas without scaling the artwork.
    pub fn resize_canvas(&mut self, w: u32, h: u32) {
        let nw = w.max(1) as usize;
        let nh = h.max(1) as usize;
        for layer in &mut self.layers {
            for f in &mut layer.frames {
                if f.kind != FrameKind::Key {
                    continue;
                }
                let mut nb = Bitmap::new(nw, nh);
                if let Some(old) = f.bitmap.take() {
                    nb.draw_bitmap(&old, 0, 0, 1.0);
                }
                f.bitmap = Some(nb);
            }
        }
        self.width = w.max(1);
        self.height = h.max(1);
    }

    /// Straight (non-premultiplied) RGBA of the composited document at a
    /// single pixel. Visible layers are blended over the background, matching
    /// [`Self::flatten`]. Returns `None` for coordinates outside the canvas.
    pub fn sample_pixel(&self, frame: usize, x: usize, y: usize) -> Option<[u8; 4]> {
        if x >= self.width as usize || y >= self.height as usize {
            return None;
        }
        let bg_a = self.bg[3] as f32 / 255.0;
        let mut acc = [
            self.bg[0] as f32 / 255.0 * bg_a,
            self.bg[1] as f32 / 255.0 * bg_a,
            self.bg[2] as f32 / 255.0 * bg_a,
            bg_a,
        ];
        for layer in &self.layers {
            if !layer.visible {
                continue;
            }
            let Some(bmp) = layer.bitmap_at(frame) else {
                continue;
            };
            if x >= bmp.w || y >= bmp.h {
                continue;
            }
            let i = (y * bmp.w + x) * 4;
            let px = &bmp.pixels()[i..i + 4];
            let src_a = px[3] as f32 / 255.0;
            if src_a <= 0.0 {
                continue;
            }
            let inv = 1.0 - src_a;
            for (c, &value) in px.iter().take(3).enumerate() {
                acc[c] = acc[c] * inv + value as f32 / 255.0;
            }
            acc[3] = acc[3] * inv + src_a;
        }
        if acc[3] <= 0.0 {
            return Some([0, 0, 0, 0]);
        }
        let alpha = acc[3].min(1.0);
        let unpremultiply = |v: f32| ((v / acc[3]).clamp(0.0, 1.0) * 255.0).round() as u8;
        Some([
            unpremultiply(acc[0]),
            unpremultiply(acc[1]),
            unpremultiply(acc[2]),
            (alpha * 255.0).round() as u8,
        ])
    }

    /// Flatten every visible layer at `frame` onto the document background.
    pub fn flatten(&self, frame: usize) -> Bitmap {
        let mut out = Bitmap::new(self.width as usize, self.height as usize);
        let bg = self.bg;
        if bg[3] > 0 {
            let a = bg[3] as f32 / 255.0;
            out.fill([
                (bg[0] as f32 * a) as u8,
                (bg[1] as f32 * a) as u8,
                (bg[2] as f32 * a) as u8,
                bg[3],
            ]);
        }
        for layer in &self.layers {
            if !layer.visible {
                continue;
            }
            if let Some(bmp) = layer.bitmap_at(frame) {
                out.draw_bitmap(bmp, 0, 0, 1.0);
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// File format (.tcraft)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct DocFile {
    version: u32,
    name: String,
    width: u32,
    height: u32,
    fps: f32,
    bg: [u8; 4],
    layers: Vec<LayerFile>,
}

#[derive(Serialize, Deserialize)]
struct LayerFile {
    name: String,
    visible: bool,
    locked: bool,
    frames: Vec<FrameFile>,
}

#[derive(Serialize, Deserialize)]
struct FrameFile {
    /// 0 = empty, 1 = hold, 2 = key
    kind: u8,
    /// Index into the image table for keyframes that own a bitmap.
    image: Option<u32>,
}

pub fn save_document(doc: &Document, path: &Path) -> Result<(), String> {
    let mut images: Vec<Vec<u8>> = Vec::new();
    let mut layers = Vec::new();
    for layer in &doc.layers {
        let mut frames = Vec::new();
        for f in &layer.frames {
            let (kind, image) = match f.kind {
                FrameKind::Empty => (0, None),
                FrameKind::Hold => (1, None),
                FrameKind::Key => match f.bitmap.as_ref() {
                    Some(bmp) => {
                        images.push(encode_png(bmp)?);
                        (2, Some((images.len() - 1) as u32))
                    }
                    None => (2, None),
                },
            };
            frames.push(FrameFile { kind, image });
        }
        layers.push(LayerFile {
            name: layer.name.clone(),
            visible: layer.visible,
            locked: layer.locked,
            frames,
        });
    }
    let file = DocFile {
        version: FILE_VERSION,
        name: doc.name.clone(),
        width: doc.width,
        height: doc.height,
        fps: doc.fps,
        bg: doc.bg,
        layers,
    };
    let json = serde_json::to_vec(&file).map_err(|e| format!("serialize: {e}"))?;

    let f = std::fs::File::create(path).map_err(|e| format!("create file: {e}"))?;
    let mut w = BufWriter::new(f);
    w.write_all(MAGIC).map_err(|e| e.to_string())?;
    w.write_all(&(json.len() as u32).to_le_bytes()).map_err(|e| e.to_string())?;
    w.write_all(&json).map_err(|e| e.to_string())?;
    for img in &images {
        w.write_all(&(img.len() as u32).to_le_bytes()).map_err(|e| e.to_string())?;
        w.write_all(img).map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_document(path: &Path) -> Result<Document, String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("open: {e}"))?;
    let mut magic = [0u8; 8];
    f.read_exact(&mut magic).map_err(|_| "not a TAniCraft project".to_string())?;
    if &magic != MAGIC {
        return Err("not a TAniCraft project (bad magic)".into());
    }
    let mut len4 = [0u8; 4];
    f.read_exact(&mut len4).map_err(|e| e.to_string())?;
    let json_len = u32::from_le_bytes(len4) as usize;
    if json_len > 64 * 1024 * 1024 {
        return Err("corrupt file (json too large)".into());
    }
    let mut json = vec![0u8; json_len];
    f.read_exact(&mut json).map_err(|e| e.to_string())?;
    let file: DocFile = serde_json::from_slice(&json).map_err(|e| format!("parse: {e}"))?;
    if file.version > FILE_VERSION {
        return Err(format!(
            "project uses a newer format (v{}), please update TAniCraft",
            file.version
        ));
    }

    let mut images: Vec<Bitmap> = Vec::new();
    loop {
        let mut l4 = [0u8; 4];
        match f.read_exact(&mut l4) {
            Ok(()) => {}
            Err(_) => break,
        }
        let n = u32::from_le_bytes(l4) as usize;
        let mut buf = vec![0u8; n];
        f.read_exact(&mut buf).map_err(|e| format!("read image: {e}"))?;
        images.push(decode_png(&buf).map_err(|e| format!("decode image: {e}"))?);
    }

    let mut doc = Document::new(
        file.name,
        file.width.max(1),
        file.height.max(1),
        file.fps,
        file.bg,
    );
    doc.layers.clear();
    doc.next_uid = 1;
    for lf in file.layers {
        let uid = doc.take_uid_for_load();
        let mut layer = Layer {
            uid,
            name: lf.name,
            visible: lf.visible,
            locked: lf.locked,
            frames: Vec::new(),
        };
        for ff in lf.frames {
            let frame = match ff.kind {
                0 => Frame::empty(),
                1 => Frame::hold(),
                _ => match ff.image {
                    Some(idx) => match images.get(idx as usize) {
                        Some(bmp) => Frame::key_from(bmp.clone()),
                        None => Frame::key(doc.width as usize, doc.height as usize),
                    },
                    None => Frame::key(doc.width as usize, doc.height as usize),
                },
            };
            layer.frames.push(frame);
        }
        if layer.frames.is_empty() {
            layer.frames.push(Frame::key(doc.width as usize, doc.height as usize));
        }
        doc.layers.push(layer);
    }
    if doc.layers.is_empty() {
        let uid = doc.take_uid_for_load();
        doc.layers.push(Layer::new(uid, "Layer 1", doc.width as usize, doc.height as usize));
    }
    Ok(doc)
}

impl Document {
    fn take_uid_for_load(&mut self) -> u64 {
        let uid = self.next_uid;
        self.next_uid += 1;
        uid
    }
}

fn encode_png(bmp: &Bitmap) -> Result<Vec<u8>, String> {
    let raw = bmp.to_rgba_straight();
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, bmp.w as u32, bmp.h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().map_err(|e| format!("png: {e}"))?;
        writer.write_image_data(&raw).map_err(|e| format!("png: {e}"))?;
    }
    Ok(out)
}

pub fn decode_png(bytes: &[u8]) -> Result<Bitmap, String> {
    let dec = png::Decoder::new(Cursor::new(bytes));
    let mut reader = dec.read_info().map_err(|e| e.to_string())?;
    let (w, h, color, depth) = {
        let info = reader.info();
        (info.width as usize, info.height as usize, info.color_type, info.bit_depth)
    };
    let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(w * h * 4)];
    let frame = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let data = &buf[..frame.buffer_size()];
    let mut straight = vec![0u8; w * h * 4];
    match (color, depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => straight.copy_from_slice(data),
        (png::ColorType::Rgb, png::BitDepth::Eight) => {
            for i in 0..w * h {
                straight[i * 4] = data[i * 3];
                straight[i * 4 + 1] = data[i * 3 + 1];
                straight[i * 4 + 2] = data[i * 3 + 2];
                straight[i * 4 + 3] = 255;
            }
        }
        (png::ColorType::Grayscale, png::BitDepth::Eight) => {
            for i in 0..w * h {
                let g = data[i];
                straight[i * 4] = g;
                straight[i * 4 + 1] = g;
                straight[i * 4 + 2] = g;
                straight[i * 4 + 3] = 255;
            }
        }
        _ => return Err(format!("unsupported png format {color:?}/{depth:?}")),
    }
    Ok(Bitmap::from_rgba_straight(w, h, &straight))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_pixel_uses_background_and_visible_layers() {
        let mut doc = Document::new("test", 4, 3, 24.0, [255, 255, 255, 255]);
        assert_eq!(doc.sample_pixel(0, 0, 0), Some([255, 255, 255, 255]));

        doc.layers[0].frames[0]
            .key_bitmap_mut()
            .unwrap()
            .fill([128, 0, 0, 255]);
        assert_eq!(doc.sample_pixel(0, 0, 0), Some([128, 0, 0, 255]));

        doc.layers[0].visible = false;
        assert_eq!(doc.sample_pixel(0, 0, 0), Some([255, 255, 255, 255]));
        assert_eq!(doc.sample_pixel(0, 4, 0), None);
    }

    #[test]
    fn sample_pixel_blends_semi_transparent_ink() {
        let mut doc = Document::new("test", 2, 2, 24.0, [255, 255, 255, 255]);
        doc.layers[0].frames[0]
            .key_bitmap_mut()
            .unwrap()
            .fill([128, 0, 0, 128]);
        assert_eq!(doc.sample_pixel(0, 1, 1), Some([255, 127, 127, 255]));
    }

    #[test]
    fn paste_keyframe_replaces_hold_without_changing_source_key() {
        let mut doc = Document::new("test", 2, 2, 24.0, [255, 255, 255, 255]);
        let mut copied = Bitmap::new(2, 2);
        copied.fill([128, 0, 0, 255]);
        doc.layers[0].frames[0] = Frame::key_from(copied.clone());
        doc.insert_frame(0, 0);

        assert!(doc.paste_keyframe(0, 1, copied));

        let layer = &doc.layers[0];
        assert_eq!(layer.kind_at(0), FrameKind::Key);
        assert_eq!(layer.kind_at(1), FrameKind::Key);
        assert_eq!(layer.frames[0].key_bitmap().unwrap().pixels(), &[128, 0, 0, 255].repeat(4));
        assert_eq!(layer.frames[1].key_bitmap().unwrap().pixels(), &[128, 0, 0, 255].repeat(4));
    }

    #[test]
    fn paste_keyframe_rejects_mismatched_canvas_size() {
        let mut doc = Document::new("test", 2, 2, 24.0, [255, 255, 255, 255]);
        let before = doc.layers[0].frames.clone();

        assert!(!doc.paste_keyframe(0, 1, Bitmap::new(1, 2)));
        assert_eq!(doc.layers[0].frames.len(), before.len());
        assert_eq!(doc.layers[0].kind_at(0), FrameKind::Key);
    }
}

/// Export the flattened current frame as a PNG file.
pub fn export_frame_png(doc: &Document, frame: usize, path: &Path) -> Result<(), String> {
    let bmp = doc.flatten(frame);
    let raw = bmp.to_rgba_straight();
    let f = std::fs::File::create(path).map_err(|e| format!("create: {e}"))?;
    let mut w = BufWriter::new(f);
    {
        let mut enc = png::Encoder::new(&mut w, bmp.w as u32, bmp.h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(&raw).map_err(|e| e.to_string())?;
    }
    Ok(())
}
