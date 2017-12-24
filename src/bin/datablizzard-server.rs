extern crate datablizzard;

use std::env::args;
use std::path::Path;

use datablizzard::directory::Directory;
use datablizzard::download;

fn main() {
    let mut directory = Directory::new();
    for path_str in args().skip(1) {
        let path = Path::new(&path_str);
        println!("Scanning {}", path.display());
        directory.scan_folder(path).unwrap();
    }
    println!("{:#?}", directory);
    download::server::listen("0.0.0.0:36936", &directory).unwrap();
}
