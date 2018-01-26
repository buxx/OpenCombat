# coding: utf-8
import typing

from PIL import Image
from synergine2.config import Config

from synergine2.simulation import Subject
from synergine2_cocos2d.actor import Actor
from opencombat.gui.animation import ANIMATION_WALK
from opencombat.gui.animation import ANIMATION_CRAWL
from opencombat.gui.weapon import WeaponImageApplier
from opencombat.gui.weapon import RIFFLE

FLAG_DE = 'DE'
FLAG_URSS = 'URSS'

FLAG_COLORS = {
    FLAG_DE
}

MAN_STAND_UP = 'MAN_STAND_UP'


class BaseActor(Actor):
    def __init__(
        self,
        image_path: str,
        config: Config,
        subject: Subject,
    ) -> None:
        self.weapon_image_applier = WeaponImageApplier(config, self)
        super().__init__(image_path, subject=subject, config=config)

    @property
    def mode(self) -> str:
        return MAN_STAND_UP

    @property
    def weapons(self) -> typing.List[str]:
        return []

    def get_default_appliable_images(self) -> typing.List[Image.Image]:
        if not self.weapons:
            return []

        return [
            self.weapon_image_applier.get_default_image_for_weapon(
                self.mode,
                self.weapons[0],
            )
        ]

    def firing(self) -> None:
        pass


class Man(BaseActor):
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

    @property
    def weapons(self) -> typing.List[str]:
        # TODO BS 2018-01-26: Will be managed by complex part of code
        return [RIFFLE]


class HeavyVehicle(Actor):
    animation_image_paths = {
        ANIMATION_WALK: [
            'actors/tank1.png',
        ],
        ANIMATION_CRAWL: [
            'actors/tank1.png',
        ]
    }

    def __init__(
        self,
        config: Config,
        subject: Subject,
    ) -> None:
        super().__init__('actors/tank1.png', subject=subject, config=config)
