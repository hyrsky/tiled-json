# tiled-json

[![Build Status](https://travis-ci.org/hyrsky/tiled-json.svg?branch=master)](https://travis-ci.org/hyrsky/tiled-json)

_Under development_

Read maps from the [Tiled Map Editor](http://www.mapeditor.org/) into rust. Only json files with ebedded tilesets are supported, use [rs-tiled](https://github.com/mattyhall/rs-tiled) to read xml files.

Inspired by [rs-tiled](https://github.com/mattyhall/rs-tiled) crate.

## Example

```rust
use std::path::Path;

use tiled_json::parse;

fn main() {
    let file = File::open(&Path::new("assets/map.json")).unwrap();
    println!("Opened file");
    let reader = BufReader::new(file);
    let map = parse(reader).unwrap();
    println!("{:?}", map);
}
```

## Amethyst example

Enable `json` feature with `--features`.

```rust
use amethyst::{
    assets::{AssetStorage, Asset, Handle, ProcessableAsset, ProcessingState, JsonFormat},
    ecs::{VecStorage, World},
    error::Error,
};

use tiled_json::Map;

#[derive(Clone, Debug)]
struct Tilemap(pub Map);

impl Asset for Tilemap {
    const NAME: &'static str = "tiled_json::Map";
    type Data = Map;
    type HandleStorage = VecStorage<Handle<Tilemap>>;
}

impl ProcessableAsset for Tilemap {
    fn process(data: Map) -> Result<ProcessingState<Tilemap>, Error> {
        Ok(ProcessingState::Loaded(Tilemap { 0: data }))
    }
}

fn load_tilemap(world: &mut World) -> Handle<Tilemap> {
	let loader = world.read_resource::<Loader>();
	let tilemap_store = world.read_resource::<AssetStorage<Tilemap>>();

	loader.load("assets/map.json", JsonFormat, (), &tilemap_store)
}
```

## Licences

Licenced under [MIT](LICENSE).

### Assets

[assets/tilesheet.png](assets/tilesheet.png) by Andre Mari Coppola

- itch.io dassets page: http://toen.itch.io/toens-medieval-strategy
- License - http://creativecommons.org/licenses/by/4.0/
