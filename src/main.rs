#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use std::env;
use std::fs;
use std::io;
use std::path;

mod objectstore;
mod codec;
mod hash;

fn main() -> io::Result<()> {
    let path_arg = std::env::args().nth(1).unwrap();
    let path = path::PathBuf::from(path_arg);
    let buffer_size:usize  = 1024 * 1024;

    let cwd = env::current_dir()?;
    let object_path = cwd.join("objects");
    fs::create_dir_all(&object_path)?;
    let objstore = objectstore::ObjectStore::new(object_path);

    let metadata = fs::metadata(&path).unwrap();
    if metadata.is_dir() {
        let hexdigest = objstore.write_tree(&path, buffer_size).unwrap();
        objstore.read_blob(hexdigest, &path.with_extension("decompressed"), buffer_size)?;
    } else if metadata.is_file() {
        let hexdigest = objstore.write_blob_all(&path)?;
        // let hexdigest = objstore.write_blob(&path, buffer_size)?;
        //objstore.read_blob(hexdigest, &path.with_extension("decompressed"), buffer_size)?;
    }
    Ok(())
}


// fn parse_size(size_str: &str) -> Option<usize> {
//     let value_str = size_str.trim_end();
//     let (value, unit) = value_str.split_at(value_str.len() - 2);
// 
//     let parsed_value = value.parse::<usize>().ok()?;
//     match unit {
//         "KB" => Some(parsed_value * 1024),
//         "MB" => Some(parsed_value * 1024 * 1024),
//         "GB" => Some(parsed_value * 1024 * 1024 * 1024),
//         _ => None,
//     }
// }

// TODO: commit tommorrow

