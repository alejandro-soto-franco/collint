"""pytest plugin exposing the `assert_no_collisions` fixture."""

import pytest

import collint


@pytest.fixture
def assert_no_collisions():
    def _assert(fig, **kwargs):
        report = collint.check(fig, **kwargs)
        assert not report.has_collisions, (
            f"collint found {len(report.collisions)} collisions: "
            f"{report.to_json()}"
        )

    return _assert
