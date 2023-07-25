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
    let file_path_arg = std::env::args().nth(1).unwrap();
    let buffer_size_arg = std::env::args().nth(2).unwrap();

    let file_path = path::PathBuf::from(file_path_arg);
    let buffer_size = parse_size(&buffer_size_arg);

    let cwd = env::current_dir()?;
    let object_path = cwd.join("objects");
    fs::create_dir_all(&object_path)?;

    let objstore = objectstore::ObjectStore::new(object_path);

    let hexdigest = objstore.write_blob(&file_path, buffer_size.unwrap())?;
    objstore.read_blob(&hexdigest, &file_path.with_extension("decompressed"), buffer_size.unwrap())?;
    Ok(())
}


fn parse_size(size_str: &str) -> Option<usize> {
    let value_str = size_str.trim_end();
    let (value, unit) = value_str.split_at(value_str.len() - 2);

    let parsed_value = value.parse::<usize>().ok()?;
    match unit {
        "KB" => Some(parsed_value * 1024),
        "MB" => Some(parsed_value * 1024 * 1024),
        "GB" => Some(parsed_value * 1024 * 1024 * 1024),
        _ => None,
    }
}