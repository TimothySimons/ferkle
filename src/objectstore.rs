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

impl ObjectStore {
    pub(crate) const fn new(location: path::PathBuf) -> Self {
        Self { location }
    }

    pub(crate) fn write_blob(&self, file_path: &path::PathBuf, buffer_size: usize) -> io::Result<hash::HexDigest> {
        let (temp_file, temp_file_path) = create_temp_file()?;
        let mut encoder = codec::Encoder::new(temp_file);
        let mut hasher = hash::Hasher::new();

        let mut file = fs::File::open(file_path)?;
        let size = file.metadata()?.len();
        let header = format!("blob {size}\0");
        encoder.write_all(header.as_bytes())?;
        hasher.update(header.as_bytes());

        let mut buffer = vec![0; buffer_size];
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            let buffer_slice = &buffer[..bytes_read];
            encoder.write_all(buffer_slice)?;
            hasher.update(buffer_slice);
        }
        encoder.finish()?;
        let hexdigest = hasher.finish();

        let obj_path = hexdigest.to_string();
        let (obj_dir_name, obj_file_name) = obj_path.split_at(OBJ_DIR_LEN);
        let obj_dir_path = self.location.join(obj_dir_name);
        if !obj_dir_path.exists() {
            fs::create_dir(&obj_dir_path)?;
        }
        let obj_file_path = obj_dir_path.join(obj_file_name);
        fs::rename(temp_file_path, obj_file_path)?;

        Ok(hexdigest)
    }




    pub(crate) fn read_blob(&self, hexdigest: &HexDigest, file_path: &path::PathBuf, buffer_size: usize) -> io::Result<()> {
        let obj_path = hexdigest.to_string();
        let (obj_dir_name, obj_file_name) = obj_path.split_at(OBJ_DIR_LEN);
        let obj_dir_path = self.location.join(obj_dir_name);
        let obj_file_path = obj_dir_path.join(obj_file_name);

        let file = fs::File::open(obj_file_path)?;
        let mut decoder = codec::Decoder::new(file);
        let mut hasher = hash::Hasher::new();

        let mut buffer = vec![0; buffer_size];
        let mut output_file = fs::File::create(file_path)?;

        loop {
            let bytes_read = decoder.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            let buffer_slice = &buffer[..bytes_read];
            hasher.update(buffer_slice);
            output_file.write_all(buffer_slice)?;
        }

        let decoded_hexdigest = hasher.finish();
        if decoded_hexdigest != *hexdigest {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Decompressed content does not match the provided hexdigest.",
            ));
        }

        Ok(())
    }
}

fn create_temp_file() -> io::Result<(fs::File, std::path::PathBuf)>{
    let temp_dir = env::temp_dir();
    let uuid_file_name = format!("{}", Uuid::new_v4());
    let uuid_file_path = temp_dir.join(uuid_file_name);
    let temp_file = fs::File::create(&uuid_file_path)?;
    Ok((temp_file, uuid_file_path))
}

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