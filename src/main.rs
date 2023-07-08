#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use std::env;
use std::fs;
use std::io;

mod objectstore;

fn main() -> io::Result<()> {
    let cwd = env::current_dir()?;
    let object_path = cwd.join("objects");
    fs::create_dir_all(&object_path)?;
    let objstore = objectstore::ObjectStore::new(object_path);
    let filepath = cwd.join("somethingorother.txt");
    objstore.write_blob(filepath, 8 * 1024)?;
    Ok(())
}