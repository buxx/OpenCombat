# OpenCombat

Open source close combat inspired game

# Install

OS require: build-essential libsdl1.2debian libsdl-image1.2 libsdl-image1.2-dev libsdl-ttf2.0-0 libsdl-ttf2.0-dev libsdl-mixer1.2 libsdl-mixer1.2-dev

Note: OpenCombat is in development mode. 

Considering in your virtual environment, install synergine2:

    git clone https://github.com/buxx/synergine2.git
    cd synergine2
    python setup.py develop
    pip install -e ".[cocos2d]"

Then install additional or specific version of development environment:

    pip install -r requirements.txt

You also need a running redis server (used db number is `0`, soon configurable). 

# Run

Example run:

    python run.py opencombat/maps/003

# Actual keys

When unit selected: 

* R: run
* C: crouch
* M: move
* F: fire (not implemented)
