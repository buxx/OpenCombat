# coding: utf-8
from synergine2.config import Config

from opencombat.strategy.unit.stash import UnitStash


def test_units_stash__ok__instantiate(
    config: Config,
):
    UnitStash(
        config,
        'opencombat/strategy/units.xml',
    )
