# coding: utf-8
import pytest
from synergine2.config import Config

from opencombat.exception import NotFoundException
from opencombat.strategy.team.model import TeamModel
from opencombat.strategy.team.stash import TeamStash
from opencombat.strategy.unit.model import UnitModel
from opencombat.strategy.unit.stash import UnitStash


@pytest.fixture
def unit_stash(
    config: Config,
) -> UnitStash:
    return UnitStash(
        config,
        'tests/fixtures/units.xml',
    )


def test_units_stash__ok__instantiate(
    config: Config,
    unit_stash: UnitStash,
):
    TeamStash(
        config,
        'opencombat/strategy/teams.xml',
        unit_stash=unit_stash,
    )


def test_team_stash__ok__get_teams(
    config: Config,
    unit_stash: UnitStash,
):
    stash = TeamStash(
        config,
        'tests/fixtures/teams.xml',
        unit_stash=unit_stash,
    )
    assert stash.teams
    assert 2 == len(stash.teams)
    assert isinstance(stash.teams[0], TeamModel)
    assert isinstance(stash.teams[1], TeamModel)

    assert 'std_team' == stash.teams[0].id
    assert 'Standard team' == stash.teams[0].name
    assert 'URSS' == stash.teams[0].country
    assert stash.teams[0].units
    assert 4 == len(stash.teams[0].units)
    assert isinstance(stash.teams[0].units[0], UnitModel)
    assert 'std_soldier' == stash.teams[0].units[0].id
    assert 'std_soldier' == stash.teams[0].units[1].id
    assert 'std_soldier' == stash.teams[0].units[2].id
    assert 'std_soldier' == stash.teams[0].units[3].id

    assert 'std_team' == stash.teams[1].id
    assert 'Standard team' == stash.teams[1].name
    assert 'DE' == stash.teams[1].country
    assert stash.teams[0].units
    assert 4 == len(stash.teams[1].units)
    assert isinstance(stash.teams[1].units[0], UnitModel)
    assert 'std_soldier' == stash.teams[1].units[0].id
    assert 'std_soldier' == stash.teams[1].units[1].id
    assert 'std_soldier' == stash.teams[1].units[2].id
    assert 'std_soldier' == stash.teams[1].units[3].id


def test_teams_stash__ok__get_unit(
    config: Config,
    unit_stash: UnitStash,
):
    stash = TeamStash(
        config,
        'tests/fixtures/teams.xml',
        unit_stash=unit_stash,
    )
    assert stash.get_team('std_team', 'URSS')


def test_teams_stash__error__get_team_wrong_country(
    config: Config,
    unit_stash: UnitStash,
):
    stash = TeamStash(
        config,
        'tests/fixtures/teams.xml',
        unit_stash=unit_stash,
    )

    with pytest.raises(NotFoundException):
        stash.get_team('std_team', 'UNKNOWN')


def test_teams_stash__error__get_team_wrong_id(
    config: Config,
    unit_stash: UnitStash,
):
    stash = TeamStash(
        config,
        'tests/fixtures/teams.xml',
        unit_stash=unit_stash,
    )

    with pytest.raises(NotFoundException):
        stash.get_team('unknown', 'URSS')
