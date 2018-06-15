# OpenCombat

[![Build Status](https://travis-ci.org/buxx/OpenCombat.svg?branch=master)](https://travis-ci.org/buxx/OpenCombat) [![Coverage Status](https://coveralls.io/repos/github/buxx/OpenCombat/badge.svg?branch=master)](https://coveralls.io/github/buxx/OpenCombat?branch=master) [![Codacy Badge](https://api.codacy.com/project/badge/Grade/917ff3fc2e184dd5a001c4571d5c583f)](https://www.codacy.com/app/sevajol.bastien/OpenCombat?utm_source=github.com&amp;utm_medium=referral&amp;utm_content=buxx/OpenCombat&amp;utm_campaign=Badge_Grade) [![Known Vulnerabilities](https://snyk.io/test/github/buxx/opencombat/badge.svg?targetFile=requirements.txt)](https://snyk.io/test/github/buxx/opencombat?targetFile=requirements.txt)

Open source close combat inspired game. Presentation here: http://www.closecombatseries.net/CCS/modules.php?name=Forums&file=viewtopic&t=11696

**Important note**: OpenCombat is in development and is developed under linux. Windows support is planned but not actually tested.

# Install

Tested only under linux, debian/Ubuntu. Before install project, install OS packages:

    build-essential libsdl1.2debian libsdl-image1.2 libsdl-image1.2-dev libsdl-ttf2.0-0 libsdl-ttf2.0-dev libsdl-mixer1.2 libsdl-mixer1.2-dev redis-server

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

Example run:

    python run.py maps/001 --state maps/001/state1.xml

# Actual keys

When unit selected: 

* R: run
* C: crouch
* M: move
* F: fire (not implemented)
