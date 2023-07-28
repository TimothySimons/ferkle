use std::io::{self, Read, Write};

use flate2::write::DeflateEncoder;
use flate2::read::DeflateDecoder;
use flate2::Compression;

#[allow(clippy::redundant_pub_crate)]
pub(crate) struct Encoder<W: Write> {
    inner: DeflateEncoder<W>,
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) struct Decoder<R: Read> {
    inner: DeflateDecoder<R>,
}

impl<R: Read> Decoder<R> {
    pub(crate) fn new(r: R) -> Self {
        let inner = DeflateDecoder::new(r);
        Self { inner }
    }

    pub(crate) fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.inner.read(buffer)?;
        Ok(bytes_read)
    }
}

impl<W: Write> Encoder<W> {
    pub(crate) fn new(w: W) -> Self {
        let inner = DeflateEncoder::new(w, Compression::default());
        Self { inner }
    }

    pub(crate) fn update(&mut self, buffer: &[u8]) -> io::Result<()> {
        self.inner.write_all(buffer)?;
        Ok(())
    }

    pub(crate) fn finish(self) -> io::Result<()> {
        self.inner.finish()?;
        Ok(())
    }
}

