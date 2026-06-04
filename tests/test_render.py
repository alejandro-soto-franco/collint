import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
import pytest

from collint.render import capture_rgba, ensure_drawn, hidden


def test_capture_returns_hwc_uint8():
    fig, ax = plt.subplots()
    ax.plot([0, 1], [0, 1])
    ensure_drawn(fig)
    buf = capture_rgba(fig)
    assert buf.dtype == np.uint8
    assert buf.ndim == 3 and buf.shape[2] == 4
    plt.close(fig)


def test_hidden_restores_visibility_after_block():
    fig, ax = plt.subplots()
    leg = ax.legend([ax.plot([0, 1], [0, 1])[0]], ["a"])
    assert leg.get_visible()
    with hidden([leg]):
        assert not leg.get_visible()
    assert leg.get_visible()
    plt.close(fig)


def test_hidden_restores_visibility_on_exception():
    fig, ax = plt.subplots()
    leg = ax.legend([ax.plot([0, 1], [0, 1])[0]], ["a"])
    with pytest.raises(RuntimeError):
        with hidden([leg]):
            assert not leg.get_visible()
            raise RuntimeError("boom")
    assert leg.get_visible()
    plt.close(fig)
