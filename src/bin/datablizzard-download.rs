extern crate datablizzard;

use std::env::args;
use std::path::Path;
use std::fs::File;

use datablizzard::manifest::Manifest;
use datablizzard::download;

fn main() {
    let mut args = args().skip(1);
    let manifest_path_str = args.next().unwrap();
    let out_path_str = args.next().unwrap();

    let manifest_path = Path::new(&manifest_path_str);
    let mut manifest_file = File::open(manifest_path).unwrap();
    let manifest = Manifest::from_read(&mut manifest_file).unwrap();
    println!("{:#?}", manifest);

    let out_path = Path::new(&out_path_str);
    download::client::download("127.0.0.1:36936", &manifest, &out_path).unwrap();
}
