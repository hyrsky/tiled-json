# tiled-json

[![Build Status](https://travis-ci.org/hyrsky/tiled-json.svg?branch=master)](https://travis-ci.org/hyrsky/tiled-json)

_Under development_

Read maps from the [Tiled Map Editor](http://www.mapeditor.org/) into rust. Only json files with ebedded tilesets are supported, use [rs-tiled](https://github.com/mattyhall/rs-tiled) to read xml files.

Inspired by [rs-tiled](https://github.com/mattyhall/rs-tiled) crate.

## Example

```rust
use std::path::Path;

use tiled::parse;

fn main() {
    let file = File::open(&Path::new("assets/map.json")).unwrap();
    println!("Opened file");
    let reader = BufReader::new(file);
    let map = parse(reader).unwrap();
    println!("{:?}", map);
}
```

## Licences

Licenced under [MIT](LICENSE).

### Assets

[assets/tilesheet.png](assets/tilesheet.png) by Andre Mari Coppola

- itch.io dassets page: http://toen.itch.io/toens-medieval-strategy
- License - http://creativecommons.org/licenses/by/4.0/
