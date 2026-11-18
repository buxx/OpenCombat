# `bevy_heightmap`


[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Katsutoshii/bevy_heightmap#license)
[![Crates.io](https://img.shields.io/crates/v/bevy_heightmap.svg)](https://crates.io/crates/bevy_heightmap)
[![Docs](https://docs.rs/bevy_heightmap/badge.svg)](https://docs.rs/bevy_heightmap/latest/bevy_heightmap/)

Load height map PNGs as meshes in `bevy`.
| Height map                           | Generated                          |
|--------------------------------------|----------------------------------------|
|![example_image](docs/terrain.hmp.png)|![example_image](docs/example_image.png)|

## Usage

Create a height map from a value function:

```rust
use bevy::prelude::*;
use bevy_heightmap::*;
let heightmap = ValueFunctionHeightMap(|p: Vec2| ((20. * p.x).sin() + (20. * p.y).sin()) / 2.);
let mesh: Mesh = heightmap.build_mesh(UVec2::new(10, 10));
assert_eq!(mesh.count_vertices(), 4 * 10 * 10);
```

Load a height map as a mesh from an image (requires `.hmp.png` extension):

```rust
use bevy::prelude::*;
use bevy_heightmap::*;
fn setup(asset_server: Res<AssetServer>) {
    let mesh: Handle<Mesh> = asset_server.load("textures/terrain.hmp.png");
}
```


## Examples

```
cargo run --example image
```

## Bevy support table

| bevy | bevy_heightmap |
| ---- | -------------- |
| 0.17 | 0.7.0          |
| 0.16 | 0.4.1-0.6.1    |
| 0.15 | 0.3.0          |
| 0.14 | 0.2.0          |
| 0.13 | 0.1.0          |
