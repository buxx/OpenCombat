# coding: utf-8
import pytest
from synergine2.config import Config


@pytest.fixture()
def config() -> Config:
    config_ = Config()
    config_.load_yaml('test_config.yaml')
    return config_
