use crate::error::{LauncherError, LauncherResult};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The SHA-256 of a file as lowercase hex. Read in chunks so big files don't fill the memory.
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;

        if read == 0 {
            break;
        }

        hasher.update(&buffer[..read]);
    }

    Ok(to_hex(&hasher.finalize()))
}

/// Reads a .sha256 file. Both "<hash>" and "<hash>  file.zip" are accepted.
pub fn parse_checksum_file(text: &str) -> Option<String> {
    let token = text.split_whitespace().next()?.to_lowercase();

    if token.len() == 64 && token.chars().all(|letter| letter.is_ascii_hexdigit()) {
        Some(token)
    } else {
        None
    }
}

pub fn verify_file(path: &Path, expected: &str) -> LauncherResult<()> {
    let actual = sha256_file(path)?;

    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(LauncherError::Checksum { expected: expected.to_lowercase(), actual })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ABC_HASH: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    fn temp_file(name: &str, content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("flux_launcher_{name}_{}", std::process::id()));

        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn the_hash_of_a_known_text_is_right() {
        let path = temp_file("hash", "abc");

        assert_eq!(sha256_file(&path).unwrap(), ABC_HASH);
        assert!(verify_file(&path, &ABC_HASH.to_uppercase()).is_ok());
    }

    #[test]
    fn a_wrong_checksum_is_an_error() {
        let path = temp_file("wrong", "abc");

        assert!(matches!(verify_file(&path, &"0".repeat(64)), Err(LauncherError::Checksum { .. })));
    }

    #[test]
    fn checksum_files_can_have_a_file_name_after_the_hash() {
        assert_eq!(parse_checksum_file(&format!("{ABC_HASH}  flux.zip\n")).as_deref(), Some(ABC_HASH));
        assert_eq!(parse_checksum_file(ABC_HASH).as_deref(), Some(ABC_HASH));
        assert!(parse_checksum_file("not a hash").is_none());
        assert!(parse_checksum_file("").is_none());
    }
}
