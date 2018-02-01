# coding: utf-8
import typing

from PIL import Image
from synergine2.config import Config
from synergine2_cocos2d.util import PathManager

from opencombat.exception import UnknownWeapon
from opencombat.gui.const import MAN_STAND_UP
from opencombat.gui.const import MAN_CRAWLING

if typing.TYPE_CHECKING:
    from opencombat.gui.actor import BaseActor

RIFFLE = 'RIFFLE'


class ImageApplier(object):
    pass  # FIXME: refact here


class WeaponImageApplier(ImageApplier):
    def __init__(
        self,
        config: Config,
        actor: 'BaseActor',
    ) -> None:
        self.actor = actor
        self._images_scheme = self.get_images_scheme()
        self.path_manager = PathManager(config.resolve('global.include_path.graphics'))
        self._cache = {}  # type: typing.Dict[str, Image.Image]

    def get_images_scheme(self) -> typing.Dict[str, typing.Dict[str, str]]:
        return {
            MAN_STAND_UP: {
                RIFFLE: [
                    'actors/man_weap1.png'
                ],
            },
            # FIXME NOW
            # MAN_CRAWLING: {
            'CRAWL': {
                RIFFLE: [
                    'actors/man_c1_weap1.png',
                    'actors/man_c2_weap1.png',
                    'actors/man_c3_weap1.png',
                    'actors/man_c4_weap1.png',
                ],

            }
        }

    def get_default_image_for_weapon(self, mode: str, weapon_type: str) -> Image.Image:
        try:
            image_file_path = self.path_manager.path(
                self._images_scheme[mode][weapon_type][0],
            )
            try:
                return self._cache[image_file_path]
            except KeyError:
                self._cache[image_file_path] = Image.open(image_file_path)
                return self._cache[image_file_path]
        except KeyError:
            raise UnknownWeapon(
                'Unknown weapon "{}" for mode "{}"'.format(weapon_type, mode),
            )

    def get_animation_image_for_weapon(
        self,
        mode: str,
        weapon_type: str,
        animation_position: int,
    ) -> Image.Image:
        try:
            # FIXME Cache
            image_file_path = self.path_manager.path(
                self._images_scheme[mode][weapon_type][animation_position],
            )
            return Image.open(image_file_path)
        except KeyError:
            raise UnknownWeapon(
                'Unknown weapon "{}" for mode "{}"'.format(weapon_type, mode),
            )

