extern crate bytebarrage;

use std::env::args;
use std::path::Path;
use std::fs::File;
use std::io::Write;

use bytebarrage::manifest::Manifest;

fn main() {
    for path_str in args().skip(1) {
        let path = Path::new(&path_str);
        let mut extension = path.extension().unwrap_or_default().to_owned();
        if extension.len() > 0 {
            extension.push(".");
        }
        extension.push("blizzard");
        let manifest_path = path.with_extension(extension);
        let manifest = Manifest::for_file(path).unwrap();
        let manifest_bytes = manifest.to_vec().unwrap();
        // println!("{:#?}", manifest);
        let mut manifest_file = File::create(manifest_path).unwrap();
        manifest_file.write_all(&manifest_bytes).unwrap();
    }
}
