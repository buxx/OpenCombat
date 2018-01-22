# coding: utf-8
import pyglet
from synergine2.config import Config

from synergine2.simulation import Subject
from synergine2_cocos2d.actor import Actor
from opencombat.gui.animation import ANIMATION_WALK
from opencombat.gui.animation import ANIMATION_CRAWL


FLAG_DE = 'DE'
FLAG_URSS = 'URSS'

FLAG_COLORS = {
    FLAG_DE
}


class Man(Actor):
    animation_image_paths = {
        ANIMATION_WALK: [
            'actors/man.png',
            'actors/man_w1.png',
            'actors/man_w2.png',
            'actors/man_w3.png',
            'actors/man_w4.png',
            'actors/man_w5.png',
            'actors/man_w6.png',
            'actors/man_w7.png',
        ],
        ANIMATION_CRAWL: [
            'actors/man_c1.png',
            'actors/man_c2.png',
            'actors/man_c3.png',
            'actors/man_c4.png',
        ]
    }

    def __init__(
        self,
        config: Config,
        subject: Subject,
    ) -> None:
        super().__init__('actors/man.png', subject=subject, config=config)
