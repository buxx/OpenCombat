# coding: utf-8
from synergine2_xyz.map import TMXMap
from PIL import Image

from opencombat.simulation.interior import InteriorManager


def test_interior_zones__one_zone():
    map_ = TMXMap('tests/fixtures/one_interior.tmx')
    manager = InteriorManager(map_)
    interiors = manager.get_interiors()

    assert interiors
    assert isinstance(interiors, list)
    assert 1 == len(interiors)

    assert (1, 1) in interiors[0]
    assert (1, 2) in interiors[0]
    assert (1, 3) in interiors[0]
    assert (2, 1) in interiors[0]
    assert (2, 2) in interiors[0]
    assert (2, 3) in interiors[0]
    assert (3, 1) in interiors[0]
    assert (3, 2) in interiors[0]
    assert (3, 3) in interiors[0]
    assert 9 == len(interiors[0])


def test_interior_zones__two_separated_zones():
    map_ = TMXMap('tests/fixtures/two_interiors.tmx')
    manager = InteriorManager(map_)
    interiors = sorted(manager.get_interiors())

    assert interiors
    assert isinstance(interiors, list)
    assert 2 == len(interiors)

    assert (0, 1) in interiors[0]
    assert (1, 1) in interiors[0]
    assert (0, 2) in interiors[0]
    assert (1, 2) in interiors[0]
    assert (0, 3) in interiors[0]
    assert (1, 3) in interiors[0]
    assert 6 == len(interiors[0])

    assert (3, 1) in interiors[1]
    assert (4, 1) in interiors[1]
    assert (3, 2) in interiors[1]
    assert (4, 2) in interiors[1]
    assert (3, 3) in interiors[1]
    assert (4, 3) in interiors[1]
    assert 6 == len(interiors[1])


def test_interiors_zones__side_by_side_zones_with_separator():
    map_ = TMXMap('tests/fixtures/side_by_side_interiors.tmx')
    manager = InteriorManager(map_)
    interiors = sorted(manager.get_interiors())

    assert interiors
    assert isinstance(interiors, list)
    assert 2 == len(interiors)

    assert (0, 1) in interiors[0]
    assert (1, 1) in interiors[0]
    assert (0, 2) in interiors[0]
    assert (1, 2) in interiors[0]
    assert (0, 3) in interiors[0]
    assert (1, 3) in interiors[0]
    assert (2, 1) in interiors[0]
    assert (2, 2) in interiors[0]
    assert (2, 3) in interiors[0]
    assert 9 == len(interiors[0])

    assert (2, 1) in interiors[1]
    assert (2, 2) in interiors[1]
    assert (2, 3) in interiors[1]
    assert (3, 1) in interiors[1]
    assert (4, 1) in interiors[1]
    assert (3, 2) in interiors[1]
    assert (4, 2) in interiors[1]
    assert (3, 3) in interiors[1]
    assert (4, 3) in interiors[1]
    assert 9 == len(interiors[1])


def test_interiors_zones__active_zones():
    # active zones are zone where someone is in
    map_ = TMXMap('tests/fixtures/two_interiors.tmx')
    manager = InteriorManager(map_)

    interiors = manager.get_interiors(where_positions=[(0, 1)])

    assert (0, 1) in interiors[0]
    assert (1, 1) in interiors[0]
    assert (0, 2) in interiors[0]
    assert (1, 2) in interiors[0]
    assert (0, 3) in interiors[0]
    assert (1, 3) in interiors[0]
    assert 6 == len(interiors[0])

    interiors = manager.get_interiors(where_positions=[(4, 2)])

    assert 1 == len(interiors)
    assert (3, 1) in interiors[0]
    assert (4, 1) in interiors[0]
    assert (3, 2) in interiors[0]
    assert (4, 2) in interiors[0]
    assert (3, 3) in interiors[0]
    assert (4, 3) in interiors[0]
    assert 6 == len(interiors[0])

    interiors = manager.get_interiors(where_positions=[(0, 1), (4, 2)])

    assert 2 == len(interiors)
    interiors = sorted(interiors)

    assert (0, 1) in interiors[0]
    assert (1, 1) in interiors[0]
    assert (0, 2) in interiors[0]
    assert (1, 2) in interiors[0]
    assert (0, 3) in interiors[0]
    assert (1, 3) in interiors[0]
    assert 6 == len(interiors[0])

    assert (3, 1) in interiors[1]
    assert (4, 1) in interiors[1]
    assert (3, 2) in interiors[1]
    assert (4, 2) in interiors[1]
    assert (3, 3) in interiors[1]
    assert (4, 3) in interiors[1]
    assert 6 == len(interiors[1])


def test_interiors_zones__make_image_transparent__just_replace():
    map_ = TMXMap('tests/fixtures/one_interior.tmx')
    manager = InteriorManager(map_)
    interiors = manager.get_interiors()
    image = Image.open('tests/fixtures/white_40x40.png')
    after_image_bytes = Image.open('tests/fixtures/white_one_interior_40x40.png').tobytes()

    manager.update_image_for_interiors(image, interiors, 8, 8)
    assert after_image_bytes == image.tobytes()


def test_interiors_zones__make_image_complex_transparent__just_replace():
    map_ = TMXMap('tests/fixtures/one_interior.tmx')
    manager = InteriorManager(map_)
    interiors = manager.get_interiors()
    image = Image.open('tests/fixtures/complex_40x40.png')
    after_image_bytes = Image.open('tests/fixtures/complex_one_interior_40x40.png').tobytes()

    manager.update_image_for_interiors(image, interiors, 8, 8)
    assert after_image_bytes == image.tobytes()


def test_interiors_zones__make_image_corner_transparent__just_replace():
    map_ = TMXMap('tests/fixtures/corner_interior.tmx')
    manager = InteriorManager(map_)
    interiors = manager.get_interiors()
    image = Image.open('tests/fixtures/white_40x40.png')
    after_image_bytes = Image.open('tests/fixtures/white_corner_interior_40x40.png').tobytes()

    manager.update_image_for_interiors(image, interiors, 8, 8)
    assert after_image_bytes == image.tobytes()
