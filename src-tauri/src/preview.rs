//! Intervalo do loop de screencap e data URL do PNG.

pub const PREVIEW_INTERVAL_MS: u64 = 500;

pub fn png_data_url(png: &[u8]) -> String {
    format!(
        "data:image/png;base64,{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, png)
    )
}

/// Largura e altura do IHDR. `None` se o buffer não é PNG.
pub fn png_size(png: &[u8]) -> Option<(u32, u32)> {
    if png.len() < 24 {
        return None;
    }
    if png.get(0..8)? != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    if png.get(12..16)? != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(png.get(16..20)?.try_into().ok()?);
    let height = u32::from_be_bytes(png.get(20..24)?.try_into().ok()?);
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

pub fn status_copy(kind: &str, detail: Option<&str>) -> String {
    match kind {
        "connecting" => "conectando…".into(),
        "connected" => "ligado".into(),
        "disconnected" => "sem conexão".into(),
        "off" => "desligado".into(),
        "error" => format!("falhou: {}", detail.unwrap_or("erro desconhecido")),
        other => {
            format!("falhou: status {other:?} não é connecting|connected|disconnected|off|error")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;

    #[test]
    fn interval_is_two_frames_per_second() {
        assert_eq!(PREVIEW_INTERVAL_MS, 500);
    }

    #[test]
    fn png_data_url_is_base64_prefixed() {
        let bytes: Vec<u8> = (0..16).map(|_| (0u8..=255u8).fake()).collect();
        let url = png_data_url(&bytes);
        assert!(url.starts_with("data:image/png;base64,"), "{url}");
        let b64 = url.trim_start_matches("data:image/png;base64,");
        let decoded =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64).unwrap();
        assert_eq!(decoded, bytes);
    }

    fn fake_png_header(width: u32, height: u32) -> Vec<u8> {
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&width.to_be_bytes());
        png.extend_from_slice(&height.to_be_bytes());
        png
    }

    #[test]
    fn png_size_reads_ihdr() {
        let width: u32 = (320u32..3840u32).fake();
        let height: u32 = (180u32..2160u32).fake();
        assert_eq!(
            png_size(&fake_png_header(width, height)),
            Some((width, height))
        );
        assert_eq!(png_size(&[0x89, b'P', b'N', b'G']), None);
    }

    #[test]
    fn status_copy_is_portuguese() {
        assert_eq!(status_copy("connecting", None), "conectando…");
        assert_eq!(status_copy("connected", None), "ligado");
        assert_eq!(status_copy("disconnected", None), "sem conexão");
        assert_eq!(status_copy("off", None), "desligado");
        let detail: String = (8..20).fake();
        let copy = status_copy("error", Some(&detail));
        assert!(copy.starts_with("falhou: "), "{copy}");
        assert!(copy.contains(&detail), "{copy}");
    }
}
