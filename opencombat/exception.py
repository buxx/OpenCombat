# coding: utf-8


class OpenCombatException(Exception):
    pass


class UnknownWeapon(OpenCombatException):
    pass


class UnknownAnimationIndex(OpenCombatException):
    pass
