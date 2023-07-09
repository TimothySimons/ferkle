use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path;

use flate2::write::DeflateEncoder;
use flate2::read::DeflateDecoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const OBJ_DIR_LEN: usize = 2;

#[allow(clippy::redundant_pub_crate)]
pub(crate) struct ObjectStore {
    location: path::PathBuf,
}

impl ObjectStore {
    pub(crate) const fn new(location: path::PathBuf) -> Self {
        Self { location }
    }

    pub(crate) fn write_blob(&self, file_path: &path::PathBuf, buffer_size: usize) -> io::Result<String> {
        let mut file = fs::File::open(file_path)?;
        let uuid_file_name = format!("{}", Uuid::new_v4());
        let uuid_file_path = self.location.join(&uuid_file_name);
        let temporary_file = fs::File::create(&uuid_file_path)?;

        let mut encoder = DeflateEncoder::new(temporary_file, Compression::default());
        let mut hasher = Sha256::new();

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
        let digest = hasher.finalize();
        let hexdigest = format!("{digest:x}");

        let (obj_dir_name, obj_file_name) = hexdigest.split_at(OBJ_DIR_LEN);
        let obj_dir_path = self.location.join(obj_dir_name);
        if !obj_dir_path.exists() {
            fs::create_dir(&obj_dir_path)?;
        }
        let obj_file_path = obj_dir_path.join(obj_file_name);
        fs::rename(uuid_file_path, obj_file_path)?;

        Ok(hexdigest)
    }


    pub(crate) fn read_blob(&self, hexdigest: &str, file_path: &path::PathBuf, buffer_size: usize) -> io::Result<()> {
        let (obj_dir_name, obj_file_name) = hexdigest.split_at(OBJ_DIR_LEN);
        let obj_dir_path = self.location.join(obj_dir_name);
        let obj_file_path = obj_dir_path.join(obj_file_name);

        let file = fs::File::open(obj_file_path)?;
        let mut decoder = DeflateDecoder::new(file);
        let mut hasher = Sha256::new();

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

        let digest = hasher.finalize();
        let decoded_hexdigest = format!("{digest:x}");

        if decoded_hexdigest != hexdigest {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Decompressed content does not match the provided hexdigest.",
            ));
        }

        Ok(())
    }

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
}