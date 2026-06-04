import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

import collint
from collint.overlay import render_overlay


def test_overlay_writes_a_png(tmp_path):
    fig, ax = plt.subplots(figsize=(2, 2))
    ax.plot(range(50), range(50))
    ax.set_xticks(range(0, 50))
    ax.set_xticklabels([f"label{i}" for i in range(50)])
    rep = collint.check(fig)
    out = tmp_path / "overlay.png"
    render_overlay(fig, rep, str(out))
    assert out.exists() and out.stat().st_size > 0
    plt.close(fig)
