use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path;

use uuid::Uuid;

use crate::codec;
use crate::hash;
use crate::hash::HexDigest;


const OBJ_DIR_LEN: usize = 2;


#[allow(clippy::redundant_pub_crate)]
pub(crate) struct ObjectStore {
    location: path::PathBuf,
}

struct WriteObject {
    temp_path: path::PathBuf,
    encoder: codec::Encoder<fs::File>,
    hasher: hash::Hasher,
}

struct ReadObject {
    hexdigest: HexDigest,
    decoder: codec::Decoder<fs::File>,
    hasher: hash::Hasher,
}


impl WriteObject {

    fn new() -> io::Result<Self> {
        let temp_dir_path = env::temp_dir();
        let uuid_file_name = Uuid::new_v4().to_string();
        let temp_path = temp_dir_path.join(uuid_file_name);
        let file = fs::File::create(&temp_path)?;
        let encoder = codec::Encoder::new(file);
        let hasher = hash::Hasher::new();
        Ok( Self { temp_path, encoder, hasher })
    }

    fn update(&mut self, buffer: &[u8]) -> io::Result<()> {
        self.encoder.update(buffer)?;
        self.hasher.update(buffer);
        Ok(())
    }

    fn finish(self, location: &path::PathBuf) -> io::Result<HexDigest> {
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


impl ReadObject {
    fn new(location: &path::PathBuf, hexdigest: HexDigest) -> io::Result<Self> {
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
            self.hasher.update(buf_slice);
        }
        return Ok(bytes_read)
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
        let mut object = WriteObject::new()?;
        let items = fs::read_dir(dir_path)?;

        for item in items {
            let item = item?;
            let entry = if item.file_type()?.is_file() {
                let hexdigest = self.write_blob(&item.path(), buf_size)?;
                format!("blob {}\t{}", hexdigest.to_string(), item.file_name().to_string_lossy())
            } else if item.file_type()?.is_dir() {
                let hexdigest = self.write_tree(&item.path(), buf_size)?;
                format!("tree {}\t{}", hexdigest.to_string(), item.file_name().to_string_lossy())
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unknown object type encountered.",
                ));
            }; 
            object.update(entry.as_bytes())?;
        }
        object.finish(&self.location)
    }


    pub(crate) fn write_blob(&self, file_path: &path::PathBuf, buf_size: usize) -> io::Result<hash::HexDigest> {
        let mut object = WriteObject::new()?;
        let mut file = fs::File::open(file_path)?;
        let mut buf = vec![0; buf_size];

        loop {
            let bytes_read = file.read(&mut buf)?;
            if bytes_read == 0 {
                break;
            }
            let buf_slice = &buf[..bytes_read];
            object.update(buf_slice)?;
        }

        object.finish(&self.location)
    }


    pub(crate) fn read_blob(&self, hexdigest: HexDigest, file_path: &path::PathBuf, buf_size: usize) -> io::Result<()> {
        let mut object = ReadObject::new(&self.location, hexdigest)?;
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



// TODO: think deeply about string_lossy

// TODO: read_blob could maybe be write_file, 
// anyway contention about this one, figure it out later...

// TODO: create a method to generate a temporary file
// * we need errors for if file already exists
// * why not use std::env::temp_dir... it makes so much more sense

// TODO:
// * create a hash::Hasher module and a Codec::encoder & Codec::decoder module wrappers 
//   return HexDigest type, ensures size & format
//   (agnostic to underlying algorithms)
//   (benchmarking/testing *later* becomes much easier - when there are bench/test suites)
// * let hasher = Hash::new(); hasher.update(...); hasher.finish(); etc.
// * let encoder = Codec::new(); encoder.update(...); encoder.finish(); etc.
// * See wasmer/lib/cache/src/hash.rs

// TODO:
// 1. a first draft of write_blob
// 2. performance & validity testing framework
// 3. refine & optimise
// 4. clippy & perfect

// TODO:
// * flate2 is the most downloaded rust compression library
// * flate2 supports miniz_oxide (pure rust), zlib and gzip.
// * flate2 will allow us to test each of these underlying compression strategies

// TODO: 
// * pub(crate) fn write_tree(&self, directory: path::PathBuf) {}

// TODO: 
// * ?s are like todos themselves, need to revisit them and ensure best practice error-handling
//   and propogation

// TODO: 
// * consider making BUFFER_SIZE a constant after some benchmarking...

// TODO: We need to find a smarter way of handling OsStrings...