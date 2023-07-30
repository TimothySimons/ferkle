use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path;

use uuid::Uuid;

use crate::codec;
use crate::hash;
use crate::hash::HexDigest;

const OBJ_DIR_LEN: usize = 2;

struct ObjectWriter {
    temp_path: path::PathBuf,
    encoder: codec::Encoder<fs::File>,
    hasher: hash::Hasher,
}

struct ObjectReader {
    hexdigest: HexDigest,
    decoder: codec::Decoder<fs::File>,
    hasher: hash::Hasher,
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) struct ObjectStore {
    location: path::PathBuf,
}

impl ObjectWriter {

    fn new() -> io::Result<Self> {
        let temp_dir_path = env::temp_dir();
        let uuid_file_name = Uuid::new_v4().to_string();
        let temp_path = temp_dir_path.join(uuid_file_name);
        let file = fs::File::create(&temp_path)?;
        let encoder = codec::Encoder::new(file);
        let hasher = hash::Hasher::new();
        Ok( Self { temp_path, encoder, hasher })
    }

    fn write_all(&mut self, buffer: &[u8]) -> io::Result<()> {
        self.encoder.write_all(buffer)?;
        self.hasher.write_all(buffer);
        Ok(())
    }

    fn finish(self, location: &path::Path) -> io::Result<HexDigest> {
        self.encoder.finish()?;
        let hexdigest = self.hasher.finish();
        let obj_hexdigest = hexdigest.to_string();
        let (obj_dir_name, obj_file_name) = obj_hexdigest.split_at(OBJ_DIR_LEN);
        let obj_dir_path = location.join(obj_dir_name);
        if !obj_dir_path.exists() {
            fs::create_dir(&obj_dir_path)?;
        }
        let obj_file_path = obj_dir_path.join(obj_file_name);
        fs::rename(self.temp_path, obj_file_path)?;
        Ok(hexdigest)
    }
}


impl ObjectReader {
    
    fn new(location: &path::Path, hexdigest: HexDigest) -> io::Result<Self> {
        let obj_path = hexdigest.to_string();
        let (obj_dir_name, obj_file_name) = obj_path.split_at(OBJ_DIR_LEN);
        let obj_dir_path = location.join(obj_dir_name);
        let obj_file_path = obj_dir_path.join(obj_file_name);

        let file = fs::File::open(obj_file_path)?;
        let decoder = codec::Decoder::new(file);
        let hasher = hash::Hasher::new();
        Ok( Self { hexdigest, decoder, hasher })
    }

    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.decoder.read(buffer)?;
        if bytes_read != 0 {
            let buf_slice = &buffer[..bytes_read];
            self.hasher.write_all(buf_slice);
        }
        Ok(bytes_read)
    }

    fn finish(self) -> io::Result<()> {
        let hexdigest = self.hasher.finish();
        if hexdigest != self.hexdigest {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Decompressed content does not match the provided hexdigest.",
            ));
        }
        Ok(())
    }
}


impl ObjectStore {

    pub(crate) const fn new(location: path::PathBuf) -> Self {
        Self { location }
    }

    pub(crate) fn write_tree(&self, dir_path: &path::PathBuf, buf_size: usize) -> io::Result<hash::HexDigest> {
        let mut object = ObjectWriter::new()?;
        let items = fs::read_dir(dir_path)?;

        for item in items {
            let item = item?;
            let entry = if item.file_type()?.is_file() {
                let hexdigest = self.write_blob(&item.path(), buf_size)?;
                format!("blob {}\t{}\n", hexdigest.to_string(), item.file_name().to_string_lossy())
            } else if item.file_type()?.is_dir() {
                let hexdigest = self.write_tree(&item.path(), buf_size)?;
                format!("tree {}\t{}\n", hexdigest.to_string(), item.file_name().to_string_lossy())
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unknown object type encountered.",
                ));
            }; 
            object.write_all(entry.as_bytes())?;
        }
        object.finish(&self.location)
    }

    pub(crate) fn write_blob(&self, file_path: &path::PathBuf, buf_size: usize) -> io::Result<hash::HexDigest> {
        let mut object = ObjectWriter::new()?;
        let mut file = fs::File::open(file_path)?;
        let mut buf = vec![0; buf_size];
        loop {
            let bytes_read = file.read(&mut buf)?;
            if bytes_read == 0 {
                break;
            }
            let buf_slice = &buf[..bytes_read];
            object.write_all(buf_slice)?;
        }
        object.finish(&self.location)
    }


    pub(crate) fn read_blob(&self, hexdigest: HexDigest, file_path: &path::PathBuf, buf_size: usize) -> io::Result<()> {
        let mut object = ObjectReader::new(&self.location, hexdigest)?;
        let mut file = fs::File::create(file_path)?;
        let mut buf = vec![0; buf_size];
        loop {
            let bytes_read = object.read(&mut buf)?;
            if bytes_read == 0 {
                break;
            }
            let buf_slice = &buf[..bytes_read];
            file.write_all(buf_slice)?;
        }
        object.finish()
    }

}