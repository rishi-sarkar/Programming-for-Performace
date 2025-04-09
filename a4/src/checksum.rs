use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Default)]
pub struct Checksum(String);

impl Checksum {
    // Initialize the checksum with the SHA256 hash of the input string
    pub fn with_sha256(sha: &str) -> Self {
        let digest = Sha256::digest(sha.as_bytes());
        Self(hex::encode(digest.as_slice()))
    }

    // XOR the two checksums
    pub fn update(&mut self, rhs: Self) {
        if self.0.is_empty() {
            *self = rhs;
        } else if rhs.0.is_empty() {
        } else {
            let a = hex::decode(&self.0).unwrap();
            let b = hex::decode(&rhs.0).unwrap();
            assert_eq!(a.len(), b.len());

            let c = a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| x ^ y)
                .collect::<Vec<_>>();
            *self = Checksum(hex::encode(c))
        };
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
