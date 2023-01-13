# OpenCombat

![OpenCombat illustration](intro.png)

Open source close combat inspired game. Presentation available [here](http://www.closecombatseries.net/CCS/modules.php?name=Forums&file=viewtopic&t=11696)

## Development

To be able to compile, please install (Debian packages example)

    libudev-dev libzmq3-dev

To run:

    cargo run Server --server-rep-address tcp://0.0.0.0:4255 --server-bind-address tcp://0.0.0.0:4256
