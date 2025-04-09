use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Default)]
pub struct Checksum(Option<[u8; 32]>);

impl Checksum {
    // Initialize the checksum with the SHA256 hash of the input string
    pub fn with_sha256(input: &str) -> Self {
        let digest = Sha256::digest(input.as_bytes());
        let mut arr = [0u8; 32];
        arr.copy_from_slice(digest.as_slice());
        Self(Some(arr))
    }

    // XOR the two checksums
    pub fn update(&mut self, rhs: Self) {
        match (self.0.as_mut(), rhs.0) {
            (std::option::Option::None, _) => self.0 = rhs.0,
            (_, std::option::Option::None) => {},
            (Some(self_bytes), Some(rhs_bytes)) => {
                for i in 0..32 {
                    self_bytes[i] ^= rhs_bytes[i];
                }
            }
        }
    }

    // Optional: merge another checksum into self, if you prefer a more explicit name.
    pub fn merge(&mut self, other: Self) {
        self.update(other);
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(ref bytes) => write!(f, "{}", hex::encode(bytes)),
            std::option::Option::None => write!(f, ""),
        }
    }
}
