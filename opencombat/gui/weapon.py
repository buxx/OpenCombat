# coding: utf-8
import typing

from PIL import Image
from synergine2.config import Config
from synergine2_cocos2d.util import PathManager

from opencombat.exception import UnknownWeapon

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

    def get_images_scheme(self) -> typing.Dict[str, typing.Dict[str, str]]:
        from opencombat.gui.actor import MAN_STAND_UP
        return {
            MAN_STAND_UP: {
                RIFFLE: 'actors/man_weap1.png',
            }
        }

    def get_default_image_for_weapon(self, mode: str, weapon_type: str) -> Image.Image:
        try:
            # FIXME Cache
            image_file_path = self.path_manager.path(
                self._images_scheme[mode][weapon_type],
            )
            return Image.open(image_file_path)
        except KeyError:
            raise UnknownWeapon(
                'Unknown weapon "{}" for mode "{}"'.format(weapon_type, mode),
            )

