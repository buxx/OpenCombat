# coding: utf-8
import typing

import time

import pyglet
from PIL import Image
from synergine2.config import Config
from synergine2.simulation import Subject
from synergine2_cocos2d.actor import Actor

from opencombat.exception import UnknownWeapon, UnknownAnimationIndex
from opencombat.gui.animation import ANIMATION_CRAWL
from opencombat.gui.animation import ANIMATION_WALK
from opencombat.gui.const import POSITION_MAN_STAND_UP
from opencombat.gui.const import POSITION_MAN_CRAWLING
from opencombat.gui.image import TileImageCacheManager
from opencombat.gui.weapon import RIFFLE
from opencombat.gui.weapon import WeaponImageApplier

if typing.TYPE_CHECKING:
    from opencombat.gui.fire import GuiFiringEvent


class BaseActor(Actor):
    position_matching = {
        ANIMATION_WALK: POSITION_MAN_STAND_UP,
        ANIMATION_CRAWL: POSITION_MAN_CRAWLING,
    }

    def __init__(
        self,
        image_path: str,
        config: Config,
        subject: Subject,
    ) -> None:
        self._mode = POSITION_MAN_STAND_UP
        self.weapon_image_applier = WeaponImageApplier(config, self)
        super().__init__(image_path, subject=subject, config=config)

        # Firing
        self.last_firing_time = 0
        self.firing_change_image_gap = 0.05  # seconds
        self.firing_images_cache = {}  # type: TODO

    def get_image_cache_manager(self) -> TileImageCacheManager:
        return TileImageCacheManager(self, self.config)

    @property
    def mode(self) -> str:
        return self._mode

    @property
    def weapons(self) -> typing.List[str]:
        return []

    def get_default_appliable_images(self) -> typing.List[Image.Image]:
        if not self.weapons:
            return []

        return [
            self.weapon_image_applier.get_image_for_weapon(
                self.mode,
                self.weapons[0],  # FIXME
            )
        ]

    def get_animation_appliable_images(
        self,
        animation_name: str,
        animation_position: int,
    ) -> typing.List[Image.Image]:
        if not self.weapons:
            return []

        position = self.position_matching[animation_name]

        try:
            return [
                self.weapon_image_applier.get_animation_image_for_weapon(
                    position,
                    self.weapons[0],
                    animation_position,
                )
            ]
        except UnknownWeapon:
            return []

    def get_modes(self) -> typing.List[str]:
        return [POSITION_MAN_STAND_UP]

    # def build_firing_images(self) -> None:
    #     cache_dir = self.config.resolve('global.cache_dir_path')
    #     for mode in self.get_modes():
    #         for weapon in self.weapons:
    #             images = self.weapon_image_applier.get_firing_image(
    #                 mode=mode,
    #                 weapon_type=weapon,
    #             )
    #
    #             for position in range(len(images)):
    #                 pass
    #
    #
    #     for animation_name, animation_image_paths in self.animation_image_paths.items():
    #         self.animation_images[animation_name] = []
    #         for i, animation_image_path in enumerate(animation_image_paths):
    #             final_image_path = self.path_manager.path(animation_image_path)
    #             final_image = Image.open(final_image_path)
    #
    #             for appliable_image in self.get_animation_appliable_images(
    #                 animation_name,
    #                 i,
    #             ):
    #                 final_image.paste(
    #                     appliable_image,
    #                     (0, 0),
    #                     appliable_image,
    #                 )
    #
    #             # FIXME NOW: nom des image utilise au dessus
    #             final_name = '_'.join([
    #                 str(subject_id),
    #                 ntpath.basename(final_image_path),
    #             ])
    #             final_path = os.path.join(cache_dir, final_name)
    #
    #             final_image.save(final_path)
    #
    #             self.animation_images[animation_name].append(
    #                 pyglet.image.load(
    #                     final_path,
    #                 )
    #             )

    def firing(self, firing: 'GuiFiringEvent') -> None:
        # FIXME: move some code ?
        now = time.time()
        if now - self.last_firing_time >= self.firing_change_image_gap:
            self.last_firing_time = now
            firing.increment_animation_index()

            try:
                image = self.image_cache.firing_cache.get(
                    mode=self.mode,
                    weapon=firing.weapon,
                    position=firing.animation_index,
                )
            except UnknownAnimationIndex:
                image = self.image_cache.firing_cache.get(
                    mode=self.mode,
                    weapon=firing.weapon,
                    position=0,
                )
                firing.reset_animation_index()

            # FIXME cache: prepare before firing
            import uuid
            tmp_path = '/tmp/{}.png'.format(str(uuid.uuid4()))
            image.save(tmp_path)
            pyglet_image = pyglet.image.load(tmp_path)

            self.update_image(pyglet_image.get_texture())


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


class HeavyVehicle(BaseActor):
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

    @property
    def weapons(self) -> typing.List[str]:
        # TODO BS 2018-01-26: Will be managed by complex part of code
        return [RIFFLE]
