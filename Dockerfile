FROM debian:9-slim

WORKDIR /app

# Install python3 and system dependencies
RUN apt update \
    && apt -y --no-install-recommends install \
        build-essential python3-tk python3-pip \
        libsdl1.2debian libsdl-image1.2 \ 
        libsdl-image1.2-dev libsdl-ttf2.0-0 \
        libsdl-ttf2.0-dev libsdl-mixer1.2 \
        libsdl-mixer1.2-dev redis-server \
        git python3-setuptools python3-dev \ 
        python3-coverage python3-wheel

# Install opencombat dependencies external
ADD requirements.txt /app/requirements.txt
RUN pip3 install -r requirements.txt

# Install opencombat dependencies from github
RUN git clone https://github.com/buxx/synergine2.git \
    && cd synergine2 \
    && python3 setup.py develop \
    && pip3 install -e ".[cocos2d]"

WORKDIR /app/open_combat
COPY . /app/open_combat
