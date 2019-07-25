use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use tiled_json::parse;

fn main() {
    let file = File::open(&Path::new("assets/map.json")).unwrap();
    println!("Opened file");
    let reader = BufReader::new(file);
    let map = parse(reader).unwrap();
    println!("{:?}", map);
}
