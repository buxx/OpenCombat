setup:
    # cargo install cargo-all-features
    cargo binstall cargo-nextest --secure

check:
    cargo check
    cargo check --tests
    cargo check --features debug,perfs,tracker,test
    cargo clippy

make-caches:
    cargo build
    cargo build --features debug,perfs
    cargo build --features test
    cargo build --tests
    cargo build --release
    just check

serve-minidblue:
    cargo run --bin oc_world_server --features debug -- \
      ./mods/std1 ./examples/minidblue --snapshot ./examples/minidblue.snapshot

serve-world1:
    cargo run --bin oc_world_server --features debug -- \
      ./mods/std1 ./examples/world1 --snapshot ./examples/world1.snapshot

gui *args:
    cargo run --bin oc_battle_gui {{ args }} -- network 127.0.0.1:6589

# Record battle of local server into given file (stop with Ctrl+C or at battle end)
record file *args:
    cargo run --bin oc_record -- 127.0.0.1:6589 {{ file }} {{ args }}

# Run embedded game (args like example: mod, world, deployments...) and record it
record-embedded file *args:
    cargo run --bin oc_battle_gui -- embedded {{ args }} --record {{ file }}

replay file *args:
    cargo run --bin oc_battle_gui -- replay {{ file }} {{ args }}

example-world1 *args:
    cargo run --bin oc_example --features debug -- world1 {{ args }}

example-minidblue *args:
    cargo run --bin oc_example -- minidblue {{ args }}

example-height *args:
    cargo run --bin oc_example --features debug -- height {{ args }}

example-projectiles-obstacles *args:
    cargo run --bin oc_example --features debug -- projectiles-obstacles {{ args }}

example-individual-shots *args:
    cargo run --bin oc_example --features debug -- individual-shots {{ args }}

example-regions *args:
    cargo run --bin oc_example --features debug -- regions {{ args }}

example-individual-behaviors *args:
    cargo run --bin oc_example --features debug -- individual-behaviors {{ args }}

example-visibilities *args:
    cargo run --bin oc_example --features debug -- visibilities {{ args }}

example-engaging *args:
    cargo run --bin oc_example --features debug -- engaging {{ args }}

example-stress-gui-projectiles:
    cargo run --bin oc_example --features debug,perfs --release -- stress-projectiles

example-stress-server-projectiles:
    cargo run --bin oc_example --features perfs --release -- stress-projectiles-server

example-weapons *args:
    cargo run --bin oc_example --features debug -- weapons {{ args }}

example-world *args:
    cargo run --bin oc_example --features debug -- world {{ args }}

example-interiors *args:
    cargo run --bin oc_example --features debug -- interiors {{ args }}

example-stress *args:
    cargo run --bin oc_example --features debug --release -- stress {{ args }}

test:
    cargo nextest run

# speed: when not empty, run tests with OC_SPEED=100. jobs: number of tests run in parallel
test-e2e speed="" jobs="4":
    #!/usr/bin/env bash
    if [ -n "{{ speed }}" ]; then
        export OC_SPEED=100
    fi
    tests=(
        test-projectiles-obstacles-one-wall
        test-projectiles-obstacles-multiple-wall
        test-projectiles-obstacles-one-hill
        test-projectiles-obstacles-multiple-hill
        test-projectiles-obstacles-one-traverse-volumes
        test-projectiles-obstacles-one-start-inside-wall
        test-projectiles-obstacles-individual-shot-traverse-volume
        test-projectiles-obstacles-individual-shot-blocked-by-volume
        test-individual-shots-same-pixel
        test-individual-shots-in-volume
        test-individual-shots-different-tile
        test-individual-shots-above
        test-individual-shots-above-prone
        test-individual-shots-near-rotated-prone
        test-individual-behaviors-move-straight-ahead1
        test-individual-behaviors-move-straight-ahead-obstacle1
        test-individual-behaviors-move-straight-ahead2
        test-individual-behaviors-move-straight-ahead-obstacle2
        test-individual-behaviors-move-fast-straight-ahead1
        test-individual-behaviors-move-fast-straight-ahead-obstacle1
        test-individual-behaviors-move-fast-straight-ahead2
        test-individual-behaviors-move-fast-straight-ahead-obstacle2
        test-individual-behaviors-stunned
        test-individual-behaviors-throw-grenade
        test-individual-behaviors-throw-grenade-multiple
        test-individual-behaviors-throw-grenade-after-move
        test-individual-behaviors-throw-grenade-after-move-multiple
        test-individual-behaviors-throw-grenade-when-ennemy-near
        test-regions-projectile-move-out
        test-regions-projectile-move-in
        test-regions-individual-move-out
        test-regions-individual-move-in
        test-visibilities-direct
        test-visibilities-through
        test-visibilities-not-visible
        test-visibilities-hidden
        test-visibilities-discover
        test-visibilities-move-then-enemy-visible
        test-visibilities-altered-wall
        test-engaging-direct
        test-engaging-direct2
        test-engaging-far-machine-gun
        test-engaging-hedge
        test-engaging-suppressed
        test-engaging-move-to-hiding
        test-engaging-move-to-hiding-then-discover
        test-engaging-engage-one-by-one
        test-engaging-suppress-fire
        test-height-climb
        test-vehicles-tank-move-forward
        test-vehicles-tank-move-rotation
        test-vehicles-tank-target
        test-vehicles-tank-projectiles
        test-interiors-enter
        test-interiors-out
        test-interiors-outside-explosion-protected
        test-interiors-outside-explosion-not-protected
        test-interiors-on-the-top-explosion-protected
        test-interiors-on-the-top-explosion-not-protected-because-broken
        test-interiors-on-the-top-explosion-not-protected
        test-interiors-near-explosion-protected-in-bunker
        test-world-support
        test-world-end-time
        test-world-end-morale
        test-world-explosion-alter-wall
        test-world-smoke-grenade
        test-basecamp-battle-test1
        test-basecamp-battle-finished
    )
    # Build once, so parallel runs below don't compete on cargo build lock
    cargo build --bins --features test || exit 1
    just build-basecamp-e2e || exit 1
    logs=$(mktemp -d)
    trap 'rm -rf "$logs"' EXIT
    run() {
        if just "$1" >"$logs/$1.log" 2>&1; then
            echo "✅ $1"
        else
            echo "❌ $1"
            cat "$logs/$1.log"
            return 1
        fi
    }
    export -f run
    export logs
    printf '%s\n' "${tests[@]}" | xargs -P {{ jobs }} -I{} bash -c 'run {}'

# Same as test-e2e, but windows are rendered in a virtual X server (Xvfb), so not shown.
# WAYLAND_DISPLAY is removed to force X11 (Xvfb) instead of the Wayland session.
# Hardware GPU drivers can't present on Xvfb (no DRI3), so force Vulkan with Mesa software driver (lavapipe)
test-e2e-headless speed="" jobs="4":
    env -u WAYLAND_DISPLAY WGPU_BACKEND=vulkan VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json \
      xvfb-run -a just test-e2e "{{ speed }}" {{ jobs }}

test-projectiles-obstacles-one-wall:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- projectiles-obstacles one-against-wall --test

test-projectiles-obstacles-multiple-wall:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- projectiles-obstacles multiple-against-wall --test

test-projectiles-obstacles-one-hill:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- projectiles-obstacles one-against-hill --test

test-projectiles-obstacles-multiple-hill:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- projectiles-obstacles multiple-against-hill --test

test-projectiles-obstacles-one-traverse-volumes:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- projectiles-obstacles one-traverse-volumes --test

test-projectiles-obstacles-one-start-inside-wall:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- projectiles-obstacles one-start-inside-wall --test

test-projectiles-obstacles-individual-shot-traverse-volume:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- projectiles-obstacles individual-shot-traverse-volume --test

test-projectiles-obstacles-individual-shot-blocked-by-volume:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- projectiles-obstacles individual-shot-blocked-by-volume --test

test-individual-shots-same-pixel:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-shots same-pixel --test

test-individual-shots-in-volume:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-shots in-volume --test

test-individual-shots-different-tile:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-shots different-tile --test

test-individual-shots-above:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-shots above --test

test-individual-shots-above-prone:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-shots above-prone --test

test-individual-shots-near-rotated-prone:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-shots near-rotated-prone --test

test-individual-behaviors-move-straight-ahead1:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors move-straight-ahead --test --individual-count 1

test-individual-behaviors-move-straight-ahead-obstacle1:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors move-straight-ahead-obstacle --test --individual-count 1

test-individual-behaviors-move-straight-ahead2:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors move-straight-ahead --test --individual-count 2

test-individual-behaviors-move-straight-ahead-obstacle2:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors move-straight-ahead-obstacle --test --individual-count 2

test-individual-behaviors-move-fast-straight-ahead1:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors move-fast-straight-ahead --test --individual-count 1

test-individual-behaviors-move-fast-straight-ahead-obstacle1:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors move-fast-straight-ahead-obstacle --test --individual-count 1

test-individual-behaviors-move-fast-straight-ahead2:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors move-fast-straight-ahead --test --individual-count 2

test-individual-behaviors-move-fast-straight-ahead-obstacle2:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors move-fast-straight-ahead-obstacle --test --individual-count 2

test-individual-behaviors-stunned:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors stunned --test --individual-count 1

test-individual-behaviors-throw-grenade:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors throw-grenade --individual-count 1 --grenade-count 1 --test

test-individual-behaviors-throw-grenade-multiple:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors throw-grenade --individual-count 2 --grenade-count 2 --test

test-individual-behaviors-throw-grenade-after-move:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors throw-grenade-after-move --individual-count 1 --grenade-count 1 --test

test-individual-behaviors-throw-grenade-after-move-multiple:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors throw-grenade-after-move --individual-count 2 --grenade-count 2 --test

test-individual-behaviors-throw-grenade-when-ennemy-near:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- individual-behaviors throw-grenade-when-ennemy-near --individual-count 1 --grenade-count 1 --test

test-regions-projectile-move-out:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- regions projectile-move-out --test

test-regions-projectile-move-in:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- regions projectile-move-in --test

test-regions-individual-move-out:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- regions individual-move-out --test

test-regions-individual-move-in:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- regions individual-move-in --test

test-visibilities-direct:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- visibilities direct --test

test-visibilities-through:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- visibilities through --test

test-visibilities-not-visible:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- visibilities not-visible --test

test-visibilities-hidden:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- visibilities hidden --test

test-visibilities-discover:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- visibilities discover --test

test-visibilities-move-then-enemy-visible:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- visibilities move-then-enemy-visible --test

test-visibilities-altered-wall:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- visibilities altered-wall --test

test-engaging-direct:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- engaging direct --test

test-engaging-direct2:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- engaging direct2 --test

test-engaging-far-machine-gun:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- engaging far-machine-gun --test

test-engaging-hedge:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- engaging hedge --test

test-engaging-suppressed:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- engaging suppressed --test

test-engaging-move-to-hiding:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- engaging move-to-hiding --test

test-engaging-move-to-hiding-then-discover:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- engaging move-to-hiding-then-discover --test

test-engaging-engage-one-by-one:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- engaging engage-one-by-one --test

test-engaging-suppress-fire:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- engaging suppress-fire --test

test-height-climb:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- height climb --test

test-vehicles-tank-move-forward:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- vehicles tank-move-forward --test

test-vehicles-tank-move-rotation:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- vehicles tank-move-rotation --test

test-vehicles-tank-target:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- vehicles tank-target --test

test-vehicles-tank-projectiles:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- vehicles tank-projectiles --test

test-interiors-enter:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- interiors enter --test

test-interiors-out:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- interiors out --test

test-interiors-outside-explosion-protected:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- interiors outside-explosion-protected --test

test-interiors-outside-explosion-not-protected:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- interiors outside-explosion-not-protected --test

test-interiors-on-the-top-explosion-protected:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- interiors on-the-top-explosion-protected --test

test-interiors-on-the-top-explosion-not-protected-because-broken:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- interiors on-the-top-explosion-not-protected-because-broken --test

test-interiors-on-the-top-explosion-not-protected:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- interiors on-the-top-explosion-not-protected --test

test-interiors-near-explosion-protected-in-bunker:
    RUST_LOG=ERROR cargo run --bin oc_example --features test,debug -- interiors near-explosion-protected-in-bunker --test

test-world-support:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- world support --test

test-world-end-time:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- world end-time --test

test-world-end-morale:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- world end-morale --test

test-world-explosion-alter-wall:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- world explosion-alter-wall --test

test-world-smoke-grenade:
    RUST_LOG=ERROR cargo run --bin oc_example --features test -- world smoke-grenade --test

# Basecamp starts the battle gui found next to its executable (target/release)
build-for-basecamp:
    cargo build --release -p oc_basecamp -p oc_battle_gui --no-default-features
build-for-basecamp-dev:
    cargo build -p oc_basecamp -p oc_battle_gui --no-default-features

# Debug basecamp starts the debug battle gui (target/debug)
build-for-basecamp-debug:
    cargo build -p oc_battle_gui --features debug
    cargo build -p oc_basecamp --features debug

build-basecamp-e2e:
    cargo build -p oc_battle_gui --features debug
    cargo test -p oc_basecamp --features e2e --test battle --no-run

test-basecamp-battle-test1: build-basecamp-e2e
    RUST_LOG=ERROR cargo test -p oc_basecamp --features e2e --test battle -- --skip solo_battle_test1_all_flags_owned_is_finished

test-basecamp-battle-finished: build-basecamp-e2e
    RUST_LOG=ERROR cargo test -p oc_basecamp --features e2e --test battle -- --exact solo_battle_test1_all_flags_owned_is_finished

test-all:
    just test
    just test-e2e fast

test-all-headless:
    just test
    just test-e2e-headless fast

list:
    just --list
