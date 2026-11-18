# Examples

> Install [just](https://just.systems/man/en/installation.html): `cargo install just` (or your package manager) for more easy usage.

## Run

```bash
just example-<name> [scenario] [options]
```

Most examples take a scenario name and options. List them with `--help` like:

```bash
just example-engaging --help
```

Without just (or for examples without recipe):

```bash
cargo run --bin oc_example --features debug -- <name> [scenario] [options]
cargo run --bin oc_example -- --help  # list examples
```

Add `--test` to run a scenario as automated test (window closes with success/failure). See `just test-e2e`.

## List

| Example | Command | Content |
|---|---|---|
| ai-fight | `cargo run --bin oc_example --features debug -- ai-fight` | AI versus AI battle, gui only observes |
| demo2 | `cargo run --bin oc_example --features debug -- demo2` | Battle on `demo2` map |
| minidblue | `just example-minidblue` | Battle on `minidblue` map |
| world1 | `just example-world1` | Battle on `world1` map |
| engaging | `just example-engaging <scenario>` | Target choice, suppression, hiding |
| height | `just example-height <scenario>` | Height: visibility, climbing |
| individual-behaviors | `just example-individual-behaviors <scenario>` | Moves, stun, grenade throws |
| individual-shots | `just example-individual-shots <scenario>` | Shots between close individuals |
| interiors | `just example-interiors <scenario>` | Building interiors and explosion protection |
| projectiles-obstacles | `just example-projectiles-obstacles <scenario>` | Bullets against walls, hills, volumes |
| regions | `just example-regions <scenario>` | Projectiles/individuals moving across regions |
| visibilities | `just example-visibilities <scenario>` | Line of sight and fog of war |
| weapons | `just example-weapons <scenario>` | Weapons accuracy |
| world | `just example-world <scenario>` | Supports, battle end, wall alteration, smoke |
| stress | `just example-stress` | Many squads (performances) |
| stress-projectiles | `just example-stress-gui-projectiles` | Many projectiles (gui performances) |
| stress-projectiles-server | `just example-stress-server-projectiles` | Many projectiles (server performances) |
