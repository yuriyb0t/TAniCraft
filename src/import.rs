//! Импорт растровых изображений (PNG/JPEG) как `Bitmap`.
//!
//! Формат определяется по сигнатуре файла, а не по расширению. PNG декодируется
//! общим декодером из `doc`, JPEG — крейтом `jpeg-decoder`.

use std::io::Cursor;
use std::path::Path;

use jpeg_decoder::Decoder;

use crate::doc::Bitmap;

/// Читает PNG или JPEG из файла и возвращает straight-RGBA `Bitmap`.
pub fn load_image(path: &Path) -> Result<Bitmap, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read: {e}"))?;
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        crate::doc::decode_png(&bytes).map_err(|e| format!("png: {e}"))
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        decode_jpeg(&bytes)
    } else {
        Err("unsupported image format (only PNG and JPEG are supported)".to_string())
    }
}

fn decode_jpeg(bytes: &[u8]) -> Result<Bitmap, String> {
    let mut dec = Decoder::new(Cursor::new(bytes));
    let pixels = dec.decode().map_err(|e| format!("jpeg decode: {e}"))?;
    let info = dec.info().ok_or("jpeg: no image info")?;
    let (w, h) = (info.width as usize, info.height as usize);
    let mut straight = vec![0u8; w * h * 4];
    match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => {
            for i in 0..w * h {
                straight[i * 4] = pixels[i * 3];
                straight[i * 4 + 1] = pixels[i * 3 + 1];
                straight[i * 4 + 2] = pixels[i * 3 + 2];
                straight[i * 4 + 3] = 255;
            }
        }
        jpeg_decoder::PixelFormat::L8 => {
            for i in 0..w * h {
                let g = pixels[i];
                straight[i * 4] = g;
                straight[i * 4 + 1] = g;
                straight[i * 4 + 2] = g;
                straight[i * 4 + 3] = 255;
            }
        }
        jpeg_decoder::PixelFormat::CMYK32 => {
            for i in 0..w * h {
                let c = pixels[i * 4] as f32 / 255.0;
                let m = pixels[i * 4 + 1] as f32 / 255.0;
                let y = pixels[i * 4 + 2] as f32 / 255.0;
                let k = pixels[i * 4 + 3] as f32 / 255.0;
                straight[i * 4] = ((1.0 - c) * (1.0 - k) * 255.0) as u8;
                straight[i * 4 + 1] = ((1.0 - m) * (1.0 - k) * 255.0) as u8;
                straight[i * 4 + 2] = ((1.0 - y) * (1.0 - k) * 255.0) as u8;
                straight[i * 4 + 3] = 255;
            }
        }
        other => return Err(format!("unsupported jpeg pixel format {other:?}")),
    }
    Ok(Bitmap::from_rgba_straight(w, h, &straight))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::Document;

    #[test]
    fn png_import_roundtrip() {
        let doc = Document::new("t", 16, 16, 24.0, [0, 0, 0, 255]);
        let path = std::env::temp_dir().join("tanicraft_test_import.png");
        crate::doc::export_frame_png(&doc, 0, &path).unwrap();

        let bmp = load_image(&path).unwrap();
        assert_eq!(bmp.w, 16);
        assert_eq!(bmp.h, 16);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_unknown_format() {
        let path = std::env::temp_dir().join("tanicraft_test_import.txt");
        std::fs::write(&path, b"not an image").unwrap();
        assert!(load_image(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }
}
