# coding: utf-8
from synergine2.config import Config

from opencombat.strategy.team.stash import TeamStash


def test_units_stash__ok__instantiate(
    config: Config,
):
    TeamStash(
        config,
        'opencombat/strategy/teams.xml',
    )
