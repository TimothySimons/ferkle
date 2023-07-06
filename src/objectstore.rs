use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path;

use flate2::write::DeflateEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const OBJ_DIR_LEN: usize = 2;

struct ObjectStore {
    location: path::PathBuf,
}

impl ObjectStore {
    pub(crate) fn new(location: path::PathBuf) -> ObjectStore {
        fs::create_dir(location);
        ObjectStore { location }
    }

    pub(crate) fn write_blob(&self, file: fs::File, buffer_size: usize) -> io::Result<String> {
        let uuid_filename = format!("{}", Uuid::new_v4());
        let uuid_filepath = self.location.join(uuid_filename);
        let temporary_file = fs::File::create(uuid_filepath)?;

        let mut buf_reader = io::BufReader::with_capacity(buffer_size, file);
        let mut encoder = DeflateEncoder::new(temporary_file, Compression::default());
        let mut hasher = Sha256::new();

        let size = file.metadata()?.len();
        let header = format!("blob {}\0", size);
        encoder.write(header.as_bytes());
        hasher.update(header.as_bytes());
        let mut buffer = Vec::new();
        loop {
            if buf_reader.read(&mut buffer)? == 0 {
                break;
            }
            encoder.write(&buffer);
            hasher.update(&buffer);
        }
        encoder.finish();
        let digest = hasher.finalize();

        let hexdigest = format!("{:x}", digest);
        let (dir, filename) = hexdigest.split_at(OBJ_DIR_LEN);
        let path = self.location.join(dir);
        let filepath = path.join(filename);
        fs::create_dir(path);
        fs::rename(uuid_filepath, filepath);

        Ok(hexdigest)
    }
    
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
}
