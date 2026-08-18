//! Tenant seed import for `identity csu-tenant register|rotate` (Analyze-72).
//!
//! File/stdin bodies are hex text (not raw 32 bytes). Errors name the flag/path, never the body.

use std::io::{self, IsTerminal, Read};
use std::path::Path;

use anyhow::{bail, Result};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

const MAX_SEED_FILE_BYTES: u64 = 4096;

/// Trim ends, then require exactly 64 ASCII hex digits.
pub fn parse_seed_hex(raw: &str) -> Result<[u8; 32]> {
    let s = raw.trim();
    if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("--secret-hex-file must contain exactly 64 hex digits");
    }
    let bytes = hex::decode(s)
        .map_err(|_| anyhow::anyhow!("--secret-hex-file must contain exactly 64 hex digits"))?;
    bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("--secret-hex-file must contain exactly 64 hex digits"))
}

/// Read a seed from `reader`. `is_tty` is checked **before** any Read.
pub fn load_from_reader(reader: impl Read, is_tty: bool) -> Result<[u8; 32]> {
    if is_tty {
        bail!("refusing to read secret from a TTY");
    }
    let mut limited = reader.take(MAX_SEED_FILE_BYTES + 1);
    let mut buf = Vec::new();
    limited
        .read_to_end(&mut buf)
        .map_err(|_| anyhow::anyhow!("failed to read --secret-hex-file"))?;
    if buf.len() as u64 > MAX_SEED_FILE_BYTES {
        bail!("--secret-hex-file exceeds 4KiB");
    }
    let text = std::str::from_utf8(&buf)
        .map_err(|_| anyhow::anyhow!("--secret-hex-file is not valid UTF-8"))?;
    parse_seed_hex(text)
}

/// Load from `PATH`. `-` is stdin (TTY fail-closed). A file named `-` must be `./-`.
pub fn load_seed_hex_file(path: &str) -> Result<[u8; 32]> {
    if path == "-" {
        let stdin = io::stdin();
        if stdin.is_terminal() {
            bail!("refusing to read secret from a TTY");
        }
        return load_from_reader(stdin.lock(), false);
    }
    let file = std::fs::File::open(Path::new(path))
        .map_err(|_| anyhow::anyhow!("cannot open --secret-hex-file"))?;
    load_from_reader(file, false)
}

/// Frozen `--secret-hex` argv parse (C1): trim + decode, 32 bytes.
fn parse_secret_hex_argv(hex_s: &str) -> Result<SigningKey> {
    let bytes =
        hex::decode(hex_s.trim()).map_err(|e| anyhow::anyhow!("invalid --secret-hex: {e}"))?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("--secret-hex must be 32 bytes"))?;
    Ok(SigningKey::from_bytes(&arr))
}

/// File XOR argv XOR generate. Both Some → error even without clap.
pub fn resolve_tenant_signing(
    secret_hex: Option<&str>,
    secret_hex_file: Option<&str>,
) -> Result<SigningKey> {
    match (secret_hex, secret_hex_file) {
        (Some(_), Some(_)) => {
            bail!("use only one of --secret-hex or --secret-hex-file")
        }
        (None, Some(path)) => Ok(SigningKey::from_bytes(&load_seed_hex_file(path)?)),
        (Some(hex_s), None) => parse_secret_hex_argv(hex_s),
        (None, None) => Ok(SigningKey::generate(&mut OsRng)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_object::{
        reset_csu_tenants, rotate_csu_tenant_signing, save_csu_tenant_signing, AiraRef,
        CSU_TENANT_META_FILE,
    };
    use std::fs;
    use std::io::Cursor;

    const SEED_HEX: &str = "abababababababababababababababababababababababababababababababab";

    fn assert_no_seed(err: impl std::fmt::Display) {
        let s = err.to_string();
        assert!(!s.contains(SEED_HEX), "error leaked seed: {s}");
        assert!(!s.contains("abababab"), "error leaked seed fragment: {s}");
    }

    #[test]
    fn parse_accepts_newline_and_end_trim() {
        let got = parse_seed_hex(&format!("{SEED_HEX}\n")).unwrap();
        assert_eq!(got, parse_seed_hex(&format!("  {SEED_HEX}  ")).unwrap());
        assert_eq!(hex::encode(got), SEED_HEX);
    }

    #[test]
    fn parse_rejects_inner_space_prefix_length_empty() {
        assert!(
            parse_seed_hex("ab abababababababababababababababababababababababababababababab")
                .is_err()
        );
        assert!(parse_seed_hex(&format!("0x{SEED_HEX}")).is_err());
        assert!(parse_seed_hex(&SEED_HEX[..63]).is_err());
        assert!(parse_seed_hex(&format!("{SEED_HEX}a")).is_err());
        assert!(parse_seed_hex("").is_err());
        assert!(
            parse_seed_hex("gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg")
                .is_err()
        );
        let err = parse_seed_hex(&format!("0x{SEED_HEX}")).unwrap_err();
        assert_no_seed(&err);
    }

    #[test]
    fn load_from_reader_tty_and_nontty() {
        let err = load_from_reader(Cursor::new(SEED_HEX.as_bytes()), true).unwrap_err();
        assert!(err.to_string().contains("TTY"));
        assert_no_seed(&err);
        let got =
            load_from_reader(Cursor::new(format!("{SEED_HEX}\n").into_bytes()), false).unwrap();
        assert_eq!(hex::encode(got), SEED_HEX);
    }

    #[test]
    fn load_from_reader_oversize_utf8_raw() {
        let err = load_from_reader(Cursor::new(vec![b'a'; 4097]), false).unwrap_err();
        assert!(err.to_string().contains("4KiB"));
        assert_no_seed(&err);

        let err = load_from_reader(Cursor::new(vec![0xff, 0xfe, 0xfd]), false).unwrap_err();
        assert!(err.to_string().contains("UTF-8"));
        assert_no_seed(&err);

        let err = load_from_reader(Cursor::new([0xabu8; 32]), false).unwrap_err();
        assert_no_seed(&err);
    }

    #[test]
    fn load_file_ok_missing_oversize() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("seed.hex");
        fs::write(&path, format!("{SEED_HEX}\n")).unwrap();
        assert_eq!(
            hex::encode(load_seed_hex_file(path.to_str().unwrap()).unwrap()),
            SEED_HEX
        );
        let err = load_seed_hex_file(dir.path().join("nope").to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("--secret-hex-file"));
        assert_no_seed(&err);

        let huge = dir.path().join("huge.hex");
        fs::write(&huge, vec![b'a'; 4097]).unwrap();
        let err = load_seed_hex_file(huge.to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("4KiB"));
        assert_no_seed(&err);
    }

    #[test]
    fn resolve_xor_and_both_some() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("seed.hex");
        fs::write(&path, SEED_HEX).unwrap();
        let p = path.to_str().unwrap();
        let from_file = resolve_tenant_signing(None, Some(p)).unwrap();
        assert_eq!(hex::encode(from_file.to_bytes()), SEED_HEX);
        let from_argv = resolve_tenant_signing(Some(SEED_HEX), None).unwrap();
        assert_eq!(hex::encode(from_argv.to_bytes()), SEED_HEX);
        let err = resolve_tenant_signing(Some(SEED_HEX), Some(p)).unwrap_err();
        assert!(err.to_string().contains("only one"));
        assert_no_seed(&err);
        let generated = resolve_tenant_signing(None, None).unwrap();
        assert_ne!(hex::encode(generated.to_bytes()), SEED_HEX);
    }

    fn write_min_node(root: &Path, name: &str, seed: [u8; 32]) {
        let idir = root.join("identity");
        fs::create_dir_all(&idir).unwrap();
        let sk = SigningKey::from_bytes(&seed);
        let pub_hex = hex::encode(sk.verifying_key().to_bytes());
        fs::write(
            idir.join("local.ed25519"),
            format!("{}\n", hex::encode(sk.to_bytes())),
        )
        .unwrap();
        fs::write(
            idir.join("local.identity.json"),
            serde_json::json!({
                "identity_id": format!("aira:identity:{name}"),
                "identity_type": "local",
                "display_name": name,
                "public_key": { "algorithm": "ed25519", "key_hex": pub_hex },
                "created_at": "2026-08-18T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();
        fs::write(root.join("config.json"), "{}\n").unwrap();
    }

    #[test]
    fn persist_register_and_rotate_match_seed() {
        reset_csu_tenants();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_min_node(root, "cli-seed", [41u8; 32]);
        let csu = AiraRef::parse("aira:csu:cli.seed").unwrap();
        let pub_id = AiraRef::parse("aira:identity:cli-pub").unwrap();
        let seed = parse_seed_hex(SEED_HEX).unwrap();
        let sk = SigningKey::from_bytes(&seed);
        let expect = hex::encode(sk.verifying_key().to_bytes());
        let tdir = save_csu_tenant_signing(root, &csu, pub_id.clone(), sk, false).unwrap();
        let meta: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(tdir.join(CSU_TENANT_META_FILE)).unwrap())
                .unwrap();
        assert_eq!(meta["public_key_hex"].as_str().unwrap(), expect);

        let seed2 =
            parse_seed_hex("cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd")
                .unwrap();
        let sk2 = SigningKey::from_bytes(&seed2);
        let expect2 = hex::encode(sk2.verifying_key().to_bytes());
        let (_, new_pub, _, _) = rotate_csu_tenant_signing(root, &csu, sk2, false).unwrap();
        assert_eq!(new_pub, expect2);
        reset_csu_tenants();
    }
}
