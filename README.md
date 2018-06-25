# OpenCombat

[![Build Status](https://travis-ci.org/buxx/OpenCombat.svg?branch=master)](https://travis-ci.org/buxx/OpenCombat) [![Coverage Status](https://coveralls.io/repos/github/buxx/OpenCombat/badge.svg?branch=master)](https://coveralls.io/github/buxx/OpenCombat?branch=master) [![Codacy Badge](https://api.codacy.com/project/badge/Grade/917ff3fc2e184dd5a001c4571d5c583f)](https://www.codacy.com/app/sevajol.bastien/OpenCombat?utm_source=github.com&amp;utm_medium=referral&amp;utm_content=buxx/OpenCombat&amp;utm_campaign=Badge_Grade) [![Known Vulnerabilities](https://snyk.io/test/github/buxx/opencombat/badge.svg?targetFile=requirements.txt)](https://snyk.io/test/github/buxx/opencombat?targetFile=requirements.txt)

Open source close combat inspired game. Presentation here: http://www.closecombatseries.net/CCS/modules.php?name=Forums&file=viewtopic&t=11696

**Important note**: OpenCombat is in development and is developed under linux. Windows support is planned but not actually tested.

# Install

Tested only under linux, debian/Ubuntu. Before install project, install OS packages:

    build-essential python3-tk libsdl1.2debian libsdl-image1.2 libsdl-image1.2-dev libsdl-ttf2.0-0 libsdl-ttf2.0-dev libsdl-mixer1.2 libsdl-mixer1.2-dev redis-server

Python version: 3.5+

Considering in your virtual environment, install synergine2:

    git clone https://github.com/buxx/synergine2.git
    cd synergine2
    python setup.py develop
    pip install -e ".[cocos2d]"

Then install additional or specific version of development environment:

    pip install -r requirements.txt

You also need a running redis server (used db number is `0`, soon configurable). 

# Run

## Troops selection

Start troops selection GUI with:

    python select_troops.py --country USSR --country DE

Select troops for every countries then generate s troops file.

## Troops Placement phase

You must: specify a map and a state and `--placement` option:

    python run.py maps/001 --troops troops.xml --placement

`troops.xml` must be previously (at `Troops selection` phase) generated troops file.

Map will be loaded with state file troops and you will be able
to move (drag and drop with your mouse) troops.

Press `s` key will generate a state file in current dir.

## Combat phase

You must: specify a map and a state:

    python run.py maps/001 --state maps/001/state1.xml

Map will be loaded with state file troops and you will be able to order them.

# Actual keys / Give orders

When unit selected (click with mouse on soldier):

* `r`: run
* `c`: crouch
* `m`: move
* `f`: fire (not implemented)

And you can:

* `s`: Save current state into OpenCombat dir (or dir specified with `--state-save-dir`)
