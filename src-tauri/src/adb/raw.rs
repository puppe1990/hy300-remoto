//! Android 12 `screencap` stdout (no `-p`): 16-byte LE header + RGBA8888.

use super::AdbError;

const HEADER_LEN: usize = 16;
const FORMAT_RGBA_8888: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Parse HY300 `adb exec-out screencap` (Android 12 header).
///
/// ```
/// use hy300_remoto::adb::raw::parse;
/// let mut buf = Vec::new();
/// buf.extend_from_slice(&1u32.to_le_bytes());
/// buf.extend_from_slice(&1u32.to_le_bytes());
/// buf.extend_from_slice(&1u32.to_le_bytes());
/// buf.extend_from_slice(&1u32.to_le_bytes());
/// buf.extend_from_slice(&[255, 0, 0, 255]);
/// let frame = parse(&buf).unwrap();
/// assert_eq!((frame.width, frame.height), (1, 1));
/// ```
pub fn parse(bytes: &[u8]) -> Result<RawFrame, AdbError> {
    if bytes.len() < HEADER_LEN {
        return Err(AdbError::new(format!(
            "screencap raw len {}; expected >= {HEADER_LEN} header bytes (width,height,format,colorspace LE)",
            bytes.len()
        )));
    }
    let width = u32_le(bytes, 0);
    let height = u32_le(bytes, 4);
    let format = u32_le(bytes, 8);
    let payload = &bytes[HEADER_LEN..];
    let expected = pixel_bytes(width, height)?;
    if format != FORMAT_RGBA_8888 || payload.len() != expected {
        return Err(AdbError::new(format!(
            "screencap raw {width}x{height} format {format} payload {}; expected format {FORMAT_RGBA_8888} RGBA and {expected} bytes",
            payload.len()
        )));
    }
    Ok(RawFrame {
        width,
        height,
        rgba: payload.to_vec(),
    })
}

fn u32_le(bytes: &[u8], offset: usize) -> u32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes[offset..offset + 4]);
    u32::from_le_bytes(buf)
}

fn pixel_bytes(width: u32, height: u32) -> Result<usize, AdbError> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| {
            AdbError::new(format!(
                "screencap raw {width}x{height} overflow; expected pixel count that fits usize"
            ))
        })
}

/// Encode a raw RGBA frame to PNG on the Mac (device PNG is too slow during video).
///
/// ```
/// use hy300_remoto::adb::raw::{parse, to_png};
/// let mut buf = Vec::new();
/// buf.extend_from_slice(&1u32.to_le_bytes());
/// buf.extend_from_slice(&1u32.to_le_bytes());
/// buf.extend_from_slice(&1u32.to_le_bytes());
/// buf.extend_from_slice(&1u32.to_le_bytes());
/// buf.extend_from_slice(&[255, 0, 0, 255]);
/// let png = to_png(&parse(&buf).unwrap()).unwrap();
/// assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n");
/// ```
pub fn to_png(frame: &RawFrame) -> Result<Vec<u8>, AdbError> {
    let expected = pixel_bytes(frame.width, frame.height)?;
    if frame.rgba.len() != expected {
        return Err(AdbError::new(format!(
            "rgba len {}; expected {expected} for {}x{}",
            frame.rgba.len(),
            frame.width,
            frame.height
        )));
    }
    let mut out = Vec::new();
    write_png(&mut out, frame)?;
    Ok(out)
}

fn write_png(out: &mut Vec<u8>, frame: &RawFrame) -> Result<(), AdbError> {
    let mut encoder = png::Encoder::new(out, frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let mut writer = encoder.write_header().map_err(|e| {
        AdbError::new(format!(
            "png header {}x{}: {e}; expected RGBA8",
            frame.width, frame.height
        ))
    })?;
    writer.write_image_data(&frame.rgba).map_err(|e| {
        AdbError::new(format!(
            "png data {}x{}: {e}; expected {} RGBA bytes",
            frame.width,
            frame.height,
            frame.rgba.len()
        ))
    })?;
    writer.finish().map_err(|e| {
        AdbError::new(format!(
            "png finish {}x{}: {e}; expected flushed PNG",
            frame.width, frame.height
        ))
    })?;
    Ok(())
}
