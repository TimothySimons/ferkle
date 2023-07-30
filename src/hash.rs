#[allow(clippy::redundant_pub_crate)]
pub(crate) struct Hasher {
    inner: blake3::Hasher
}

#[allow(clippy::redundant_pub_crate)]
#[derive(PartialEq)]
pub(crate) struct HexDigest {
    inner: String
}

impl Hasher {
    pub(crate) fn new() -> Self {
        Self { inner: blake3::Hasher::new() }
    }

    pub(crate) fn write_all(&mut self, buffer: &[u8]) {
        self.inner.update(buffer);
    }

    pub(crate) fn finish(self) -> HexDigest {
        let digest = self.inner.finalize();
        HexDigest{ inner: digest.to_hex().to_string() }
    }
}

impl ToString for HexDigest {
    fn to_string(&self) -> String {
        self.inner.clone()
    }
}