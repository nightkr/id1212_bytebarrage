extern crate datablizzard;

use std::env::args;
use std::path::Path;

use datablizzard::manifest::Manifest;
use datablizzard::directory::Directory;

fn main() {
    let mut directory = Directory::new();
    for path_str in args().skip(1) {
        let path = Path::new(&path_str);
        println!("Scanning {}", path.display());
        directory.scan_folder(path);
    }
    println!("{:#?}", directory);
}
