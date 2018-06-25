# coding: utf-8
import pytest
from synergine2.config import Config

from opencombat.exception import NotFoundException
from opencombat.simulation.subject import ManSubject
from opencombat.strategy.team.model import UnitModel
from opencombat.strategy.unit.stash import UnitStash


def test_units_stash__ok__instantiate(
    config: Config,
):
    UnitStash(
        config,
        'opencombat/strategy/units.xml',
    )


def test_units_stash__ok__get_units(
    config: Config,
):
    stash = UnitStash(
        config,
        'tests/fixtures/units.xml',
    )
    assert stash.units
    assert 2 == len(stash.units)
    assert isinstance(stash.units[0], UnitModel)
    assert isinstance(stash.units[1], UnitModel)

    assert 'std_soldier' == stash.units[0].id
    assert 'Standard soldier' == stash.units[0].name
    assert 'USSR' == stash.units[0].country
    assert ManSubject == stash.units[0].class_

    assert 'std_soldier' == stash.units[1].id
    assert 'Standard soldier' == stash.units[1].name
    assert 'DE' == stash.units[1].country
    assert ManSubject == stash.units[1].class_


def test_units_stash__ok__get_unit(
    config: Config,
):
    stash = UnitStash(
        config,
        'tests/fixtures/units.xml',
    )
    assert stash.get_unit('std_soldier', 'USSR')


def test_units_stash__error__get_unit_wrong_country(
    config: Config,
):
    stash = UnitStash(
        config,
        'tests/fixtures/units.xml',
    )

    with pytest.raises(NotFoundException):
        stash.get_unit('std_soldier', 'UNKNOWN')


def test_units_stash__error__get_unit_wrong_id(
    config: Config,
):
    stash = UnitStash(
        config,
        'tests/fixtures/units.xml',
    )

    with pytest.raises(NotFoundException):
        stash.get_unit('unknown', 'USSR')
