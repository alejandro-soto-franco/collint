import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import pytest

import collint


def test_assert_clean_fixture_passes_on_clean(assert_no_collisions):
    fig, ax = plt.subplots()
    ax.plot([0, 1, 2], [0, 1, 0])
    assert_no_collisions(fig)
    plt.close(fig)


def test_assert_clean_fixture_raises_on_crowded(assert_no_collisions):
    fig, ax = plt.subplots(figsize=(2, 2))
    ax.plot(range(50), range(50))
    ax.set_xticks(range(0, 50))
    ax.set_xticklabels([f"label{i}" for i in range(50)])
    with pytest.raises(AssertionError):
        assert_no_collisions(fig)
    plt.close(fig)


def test_install_hook_checks_on_savefig(tmp_path, recwarn):
    collint.install()
    try:
        fig, ax = plt.subplots(figsize=(2, 2))
        ax.plot(range(50), range(50))
        ax.set_xticks(range(0, 50))
        ax.set_xticklabels([f"label{i}" for i in range(50)])
        fig.savefig(tmp_path / "out.png")
        assert any("collint" in str(w.message).lower() for w in recwarn.list)
        plt.close(fig)
    finally:
        from collint.hook import uninstall

        uninstall()
