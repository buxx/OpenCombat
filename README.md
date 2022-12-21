# OpenCombat

[![Build Status](https://travis-ci.org/buxx/OpenCombat.svg?branch=master)](https://travis-ci.org/buxx/OpenCombat) [![Coverage Status](https://coveralls.io/repos/github/buxx/OpenCombat/badge.svg?branch=master)](https://coveralls.io/github/buxx/OpenCombat?branch=master) [![Codacy Badge](https://api.codacy.com/project/badge/Grade/917ff3fc2e184dd5a001c4571d5c583f)](https://www.codacy.com/app/sevajol.bastien/OpenCombat?utm_source=github.com&amp;utm_medium=referral&amp;utm_content=buxx/OpenCombat&amp;utm_campaign=Badge_Grade) [![Known Vulnerabilities](https://snyk.io/test/github/buxx/opencombat/badge.svg?targetFile=requirements.txt)](https://snyk.io/test/github/buxx/opencombat?targetFile=requirements.txt)

Open source close combat inspired game. Presentation here: http://www.closecombatseries.net/CCS/modules.php?name=Forums&file=viewtopic&t=11696

## Dev

To be able to compile, please install (Debian packages example)

    libudev-dev libzmq3-dev

To run:

    cargo run

## Build

To build production bin:

    cargo build --release