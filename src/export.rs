//! Экспорт анимации: PNG-последовательность, анимированный GIF и видео MP4/WebM.
//!
//! GIF пишется чистым Rust (крейт `gif` с квантованием NeuQuant). Видео кодируется
//! внешним `ffmpeg` (как в Flare): кадры рендерятся в PNG-последовательность во
//! временной папке, затем ffmpeg собирает из них контейнер. Если ffmpeg не найден,
//! возвращается понятная ошибка.

use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use gif::{Encoder, Frame, Repeat};

use crate::doc::Document;

/// Порядковый номер, которым нумеруются кадры (минимум 4 цифры).
fn digits(n: usize) -> usize {
    n.to_string().len().max(4)
}

/// Экспорт всех кадров в отдельные PNG-файлы `frame_0001.png`, `frame_0002.png`, …
pub fn export_png_sequence(doc: &Document, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("create folder: {e}"))?;
    let n = doc.frame_count();
    let pad = digits(n);
    for f in 0..n {
        let name = format!("frame_{:0pad$}.png", f + 1, pad = pad);
        crate::doc::export_frame_png(doc, f, &dir.join(name))?;
    }
    Ok(())
}

/// Экспорт анимации в анимированный GIF (одна палитра на кадр, квантование NeuQuant).
pub fn export_gif(doc: &Document, path: &Path, fps: f32) -> Result<(), String> {
    let w = u16::try_from(doc.width).map_err(|_| "image too wide for GIF (max 65535 px)".to_string())?;
    let h = u16::try_from(doc.height).map_err(|_| "image too tall for GIF (max 65535 px)".to_string())?;

    let file = std::fs::File::create(path).map_err(|e| format!("create file: {e}"))?;
    let mut enc = Encoder::new(file, w, h, &[]).map_err(|e| format!("gif encoder: {e}"))?;
    enc.set_repeat(Repeat::Infinite).map_err(|e| format!("gif repeat: {e}"))?;

    // Задержка кадра в сантисекундах (10 мс), минимум 1.
    let delay = ((100.0 / fps.max(0.01)).round()).clamp(1.0, 6553.5) as u16;

    let n = doc.frame_count();
    for f in 0..n {
        let bmp = doc.flatten(f);
        let mut rgba = bmp.to_rgba_straight();
        let mut frame = Frame::from_rgba(w, h, &mut rgba);
        frame.delay = delay;
        // Каждый кадр — полное изображение; очищаем холст перед отрисовкой,
        // чтобы прозрачные пиксели не оставляли «хвосты» от предыдущего кадра.
        frame.dispose = gif::DisposalMethod::Background;
        enc.write_frame(&frame).map_err(|e| format!("gif frame {f}: {e}"))?;
    }
    Ok(())
}

/// Экспорт видео через внешний `ffmpeg`. `ext` — формат контейнера (`mp4`/`webm`),
/// он же определяет кодек и пиксельный формат.
pub fn export_video(doc: &Document, path: &Path, fps: f32, ext: &str) -> Result<(), String> {
    let Some(ffmpeg) = find_ffmpeg() else {
        return Err(
            "ffmpeg was not found on PATH. Video export needs FFmpeg.\n\
             Install it and restart TAniCraft:\n\
             \u{2022} Windows:  winget install Gyan.FFmpeg   (or download from https://ffmpeg.org)\n\
             \u{2022} macOS:    brew install ffmpeg\n\
             \u{2022} Linux:    sudo apt install ffmpeg"
                .to_string(),
        );
    };

    let n = doc.frame_count();
    let pad = digits(n);

    // Временная папка с PNG-кадрами.
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = std::env::temp_dir().join(format!(
        "tanicraft_render_{}_{}",
        std::process::id(),
        nonce
    ));
    std::fs::create_dir_all(&tmp).map_err(|e| format!("create temp folder: {e}"))?;

    let result = (|| {
        for f in 0..n {
            let name = format!("frame_{:0pad$}.png", f + 1, pad = pad);
            crate::doc::export_frame_png(doc, f, &tmp.join(name))?;
        }
        let input_pattern = tmp.join(format!("frame_%0{pad}d.png", pad = pad));

        // H.264 MP4 (совместимый yuv420p) или VP9 WebM.
        let (codec, pix_fmt): (&str, Option<&str>) = match ext {
            "webm" => ("libvpx-vp9", Some("yuva420p")),
            _ => ("libx264", Some("yuv420p")),
        };

        let mut cmd = Command::new(&ffmpeg);
        cmd.arg("-y")
            .arg("-framerate")
            .arg(format!("{fps}"))
            .arg("-i")
            .arg(&input_pattern)
            .arg("-c:v")
            .arg(codec);
        if let Some(pf) = pix_fmt {
            cmd.arg("-pix_fmt").arg(pf);
        }
        cmd.arg(path);

        let out = cmd.output().map_err(|e| format!("failed to run ffmpeg: {e}"))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(format!("ffmpeg error: {}", stderr.trim()));
        }
        Ok(())
    })();

    let _ = std::fs::remove_dir_all(&tmp);
    result
}

/// Ищет исполняемый файл ffmpeg: сначала в PATH, затем в типовом каталоге
/// установки WinGet (Windows), чтобы не зависеть от устаревшего PATH запущенного
/// терминала/Explorer.
fn find_ffmpeg() -> Option<std::path::PathBuf> {
    // 1) PATH
    if Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some(std::path::PathBuf::from("ffmpeg"));
    }

    // 2) WinGet packages: <LOCALAPPDATA>\Microsoft\WinGet\Packages\*ffmpeg*\*\bin\ffmpeg.exe
    let local = std::env::var("LOCALAPPDATA").ok()?;
    let base = std::path::Path::new(&local)
        .join("Microsoft")
        .join("WinGet")
        .join("Packages");
    for entry in std::fs::read_dir(&base).ok()?.flatten() {
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if !name.contains("ffmpeg") {
            continue;
        }
        for version_dir in std::fs::read_dir(entry.path()).ok()?.flatten() {
            let exe = version_dir.path().join("bin").join("ffmpeg.exe");
            if exe.is_file() {
                return Some(exe);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::Document;

    #[test]
    fn gif_roundtrip() {
        let mut doc = Document::new("t", 24, 16, 12.0, [255, 255, 255, 255]);
        // Второй ключевой кадр, чтобы в анимации было 2 кадра.
        doc.insert_keyframe(0, 1, true);
        assert_eq!(doc.frame_count(), 2);

        let path = std::env::temp_dir().join("tanicraft_test.gif");
        export_gif(&doc, &path, 12.0).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        let mut opts = gif::DecodeOptions::new();
        opts.set_color_output(gif::ColorOutput::RGBA);
        let mut dec = opts.read_info(std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(dec.width(), 24);
        assert_eq!(dec.height(), 16);
        let mut count = 0;
        while dec.read_next_frame().unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn png_sequence_files() {
        let mut doc = Document::new("t", 8, 8, 10.0, [0, 0, 0, 255]);
        doc.insert_keyframe(0, 1, true);
        doc.insert_keyframe(0, 2, true);
        assert_eq!(doc.frame_count(), 3);

        let dir = std::env::temp_dir().join("tanicraft_test_seq");
        export_png_sequence(&doc, &dir).unwrap();
        for f in 1..=3 {
            let p = dir.join(format!("frame_{f:04}.png"));
            assert!(p.exists(), "missing {p:?}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
