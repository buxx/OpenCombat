# OpenCombat

![OpenCombat illustration](intro.png)

Open source close combat inspired game. Presentation available [here](http://www.closecombatseries.net/CCS/modules.php?name=Forums&file=viewtopic&t=11696)

## Development

### Requirements

To be able to compile, please install (Debian packages example)

    build-essential cmake pkg-config libudev-dev libzmq3-dev

### Run

#### Standalone server

    cargo run --bin battle_server -- --bind-address 'tcp://0.0.0.0:4255' --rep-address 'tcp://0.0.0.0:4256'

#### Standalone gui

    cargo run --bin battle_gui -- --embedded-server --server-rep-address tcp://0.0.0.0:4255 --server-bind-address tcp://0.0.0.0:4256 --side a

#### Gui with embedded server

    cargo run --bin battle_gui -- --embedded-server --server-rep-address tcp://0.0.0.0:4255 --server-bind-address tcp://0.0.0.0:4256 --side a


### Profile

Install [puffin_viewer](https://github.com/EmbarkStudios/puffin/tree/main/puffin_viewer) :

    cargo install puffin_viewer

Start server or client with `--profile` flag. Example :

    cargo run --bin battle_server -- --server-rep-address tcp://0.0.0.0:4255 --server-bind-address tcp://0.0.0.0:4256 --profile

Start puffin viewer :

    puffin_viewer --url 127.0.0.1:8585

Output will be like :

![Puffin viewer](puffin_viewer.png)
