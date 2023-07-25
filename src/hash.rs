use sha2::{Digest, Sha256};

#[allow(clippy::redundant_pub_crate)]
pub(crate) struct Hasher {
    inner: Sha256
}

#[allow(clippy::redundant_pub_crate)]
#[derive(PartialEq)]
pub(crate) struct HexDigest {
    inner: String
}

impl Hasher {
    pub(crate) fn new() -> Self {
        Self { inner: Sha256::new() }
    }

    pub(crate) fn update(&mut self, buffer: &[u8]) {
        self.inner.update(buffer);
    }

    pub(crate) fn finish(self) -> HexDigest {
        let digest = self.inner.finalize();
        HexDigest{ inner: format!("{digest:x}") }
    }
}

impl ToString for HexDigest {
    fn to_string(&self) -> String {
        self.inner.clone()
    }
}