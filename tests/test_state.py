# coding: utf-8
import pytest
from synergine2.config import Config

from opencombat.exception import StateLoadError
from opencombat.state import StateLoaderBuilder, StateLoader


class MyStateLoader(StateLoader):
    pass


@pytest.fixture
def state_loader(config, simulation):
    return StateLoader(config, simulation)


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
