extern crate datablizzard;

use std::env::args;
use std::path::Path;

use datablizzard::manifest::Manifest;

fn main() {
    for path_str in args() {
        let path = Path::new(&path_str);
        let manifest = Manifest::for_file(path);
        println!("{:#?}", manifest);
    }
}
