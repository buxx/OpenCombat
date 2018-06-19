# coding: utf-8
import pytest
from synergine2.config import Config
from opencombat.simulation.base import TileStrategySimulation


@pytest.fixture()
def config() -> Config:
    config_ = Config()
    config_.load_yaml('test_config.yaml')

    config_['_runtime'] = {}
    config_['_runtime']['map_dir_path'] = 'tests/fixtures/map_a/map_a.tmx'

    return config_


@pytest.fixture
def simulation(config) -> TileStrategySimulation:
    return TileStrategySimulation(config, 'tests/fixtures/map_a/map_a.tmx')
