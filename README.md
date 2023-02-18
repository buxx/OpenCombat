# OpenCombat

![OpenCombat illustration](intro.png)

Open source close combat inspired game. Presentation available [here](http://www.closecombatseries.net/CCS/modules.php?name=Forums&file=viewtopic&t=11696)

## Development

### Requirements

To be able to compile, please install (Debian packages example)

    libudev-dev libzmq3-dev

### Start

To start Gui with embedded server :

    cargo run --bin battle_gui -- --embedded-server --server-rep-address tcp://0.0.0.0:4255 --server-bind-address tcp://0.0.0.0:4256 --side a

To start only client :

    cargo run --bin battle_gui -- --server-rep-address tcp://0.0.0.0:4255 --server-bind-address tcp://0.0.0.0:4256 --side a

To start standalone server :

    cargo run --bin battle_server -- --bind-address 'tcp://0.0.0.0:4255' --rep-address 'tcp://0.0.0.0:4256'

### Profile

Install [puffin_viewer](https://github.com/EmbarkStudios/puffin/tree/main/puffin_viewer) :

    cargo install puffin_viewer

Start server or client with `--debug` flag. Example :

    cargo run Server --server-rep-address tcp://0.0.0.0:4255 --server-bind-address tcp://0.0.0.0:4256 --debug

Start puffin viewer :

    puffin_viewer --url 127.0.0.1:8585

Output will be like :

![Puffin viewer](puffin_viewer.png)