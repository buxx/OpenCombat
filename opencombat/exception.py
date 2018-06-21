# coding: utf-8


class OpenCombatException(Exception):
    pass


class UnknownWeapon(OpenCombatException):
    pass


class UnknownFiringAnimation(OpenCombatException):
    pass


class WrongMode(OpenCombatException):
    pass


class StateLoadError(OpenCombatException):
    pass


class NotFoundError(OpenCombatException):
    pass


class NotFoundException(OpenCombatException):
    pass
