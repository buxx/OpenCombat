# coding: utf-8
import pytest
from synergine2.config import Config
from synergine2_cocos2d.const import SELECTION_COLOR_RGB

from opencombat.exception import StateLoadError
from opencombat.simulation.base import TileStrategySimulation
from opencombat.simulation.base import TileStrategySubjects
from opencombat.simulation.subject import ManSubject
from opencombat.state import StateLoaderBuilder, StateDumper
from opencombat.state import StateLoader
from opencombat.const import FLAG
from opencombat.const import SIDE
from opencombat.const import FLAG_DE
from opencombat.const import DE_COLOR
from opencombat.const import URSS_COLOR
from opencombat.const import FLAG_URSS


class MyStateLoader(StateLoader):
    pass


@pytest.fixture
def state_loader(config, simulation):
    return StateLoader(config, simulation)


@pytest.fixture
def simulation_for_dump(config) -> TileStrategySimulation:
    simulation = TileStrategySimulation(
        config,
        'tests/fixtures/map_a/map_a.tmx',
    )
    subjects = TileStrategySubjects(simulation=simulation)

    man1 = ManSubject(config, simulation)
    man1.position = (10, 11)
    man1.direction = 42
    man1.properties = {
        SELECTION_COLOR_RGB: DE_COLOR,
        FLAG: FLAG_DE,
        SIDE: 'AXIS',
    }

    man2 = ManSubject(config, simulation)
    man2.position = (16, 8)
    man2.direction = 197
    man2.properties = {
        SELECTION_COLOR_RGB: URSS_COLOR,
        FLAG: FLAG_URSS,
        SIDE: 'ALLIES',
    }

    subjects.append(man1)
    subjects.append(man2)

    return simulation


def test_state_loader_builder__ok__nominal_case(
    simulation,
):
    config = Config({
        'global': {
            'state_loader': 'tests.test_state.MyStateLoader',
        }
    })
    builder = StateLoaderBuilder(config, simulation)
    state_loader = builder.get_state_loader()
    assert type(state_loader) == MyStateLoader


def test_state_loader__ok__load_state(
    state_loader,
):
    assert state_loader.get_state('tests/fixtures/state_ok.xml')


def test_state_loader__error__state_empty(
    state_loader,
):
    with pytest.raises(StateLoadError):
        assert state_loader.get_state('tests/fixtures/state_empty.xml')


def test_state_loader__error__state_invalid(
    state_loader,
):
    with pytest.raises(StateLoadError):
        assert state_loader.get_state(
            'tests/fixtures/state_error_schema.xml',
        )


def test_state__ok__subjects(
    state_loader,
):
    state = state_loader.get_state('tests/fixtures/state_ok.xml')

    assert 2 == len(state.subjects)
    assert isinstance(state.subjects[0], ManSubject)
    assert isinstance(state.subjects[1], ManSubject)

    assert (1, 1) == state.subjects[0].position
    assert (10, 10) == state.subjects[1].position
    assert 90.0 == state.subjects[0].direction
    assert 270.0 == state.subjects[1].direction


def test_state__ok__dump(
    config: Config,
    simulation_for_dump: TileStrategySimulation,
):
    state_dumper = StateDumper(config, simulation_for_dump)
    state_xml = state_dumper.get_state_dump()
    assert False
