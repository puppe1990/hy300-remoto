//! Host ADB gravado em JSON. Padrão: HY300 neste LAN.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_HOST: &str = "192.168.15.4:3268";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostConfig {
    pub host: String,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
        }
    }
}

pub fn parse_host(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "host vazio {raw:?}, esperado host:porta (ex. {DEFAULT_HOST})"
        ));
    }
    let Some((name, port_s)) = trimmed.rsplit_once(':') else {
        return Err(format!(
            "host {trimmed:?} sem porta, esperado host:porta (ex. {DEFAULT_HOST})"
        ));
    };
    if name.is_empty() {
        return Err(format!(
            "host {trimmed:?} sem nome, esperado host:porta (ex. {DEFAULT_HOST})"
        ));
    }
    let port: u32 = port_s
        .parse()
        .map_err(|_| format!("porta {port_s:?} em {trimmed:?} não é número, esperado 1–65535"))?;
    if port < 1 || port > 65535 {
        return Err(format!("porta {port} em {trimmed:?} fora de 1–65535"));
    }
    if name
        .chars()
        .any(|c| !(c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_'))
    {
        return Err(format!(
            "host {trimmed:?} tem caractere inválido, esperado hostname ou IPv4"
        ));
    }
    Ok(format!("{name}:{port}"))
}

pub fn config_path(dir: impl AsRef<Path>) -> PathBuf {
    dir.as_ref().join("host.json")
}

pub fn load_config(dir: impl AsRef<Path>) -> HostConfig {
    let path = config_path(dir);
    let Ok(text) = fs::read_to_string(&path) else {
        return HostConfig::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save_config(dir: impl AsRef<Path>, host: &str) -> Result<HostConfig, String> {
    let parsed = parse_host(host)?;
    let cfg = HostConfig { host: parsed };
    fs::create_dir_all(dir.as_ref()).map_err(|e| format!("não criou {:?}: {e}", dir.as_ref()))?;
    let path = config_path(dir);
    let json = serde_json::to_string_pretty(&cfg).map_err(|e| format!("json: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("não gravou {}: {e}", path.display()))?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("hy300-host-{n}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn default_is_the_hy300_on_this_lan() {
        assert_eq!(HostConfig::default().host, "192.168.15.4:3268");
    }

    #[test]
    fn parse_accepts_ipv4_and_hostname() {
        assert_eq!(parse_host("  10.0.0.9:5555 ").unwrap(), "10.0.0.9:5555");
        assert_eq!(
            parse_host("BZ-CCF3059164E0.local:3268").unwrap(),
            "BZ-CCF3059164E0.local:3268"
        );
    }

    #[test]
    fn parse_rejects_empty_and_bad_port() {
        let err = parse_host("").unwrap_err();
        assert!(err.contains("vazio"), "{err}");
        assert!(err.contains("host:porta"), "{err}");

        let err = parse_host("sem-porta").unwrap_err();
        assert!(err.contains("sem-porta"), "{err}");
        assert!(err.contains("host:porta"), "{err}");

        let err = parse_host("192.168.15.4:0").unwrap_err();
        assert!(err.contains("0"), "{err}");
        assert!(err.contains("1–65535"), "{err}");

        let err = parse_host("192.168.15.4:99999").unwrap_err();
        assert!(err.contains("99999"), "{err}");
    }

    #[test]
    fn missing_file_loads_default() {
        let dir = temp_dir();
        assert_eq!(load_config(&dir), HostConfig::default());
    }

    #[test]
    fn save_then_load_roundtrip() {
        let dir = temp_dir();
        let port: u16 = (1024u16..40000u16).fake();
        let host = format!("192.168.15.4:{port}");
        save_config(&dir, &host).unwrap();
        assert_eq!(load_config(&dir).host, host);
    }

    #[test]
    fn save_rejects_invalid_and_leaves_file_alone() {
        let dir = temp_dir();
        save_config(&dir, DEFAULT_HOST).unwrap();
        let err = save_config(&dir, "sem-porta").unwrap_err();
        assert!(err.contains("sem-porta"), "{err}");
        assert_eq!(load_config(&dir).host, DEFAULT_HOST);
    }
}
