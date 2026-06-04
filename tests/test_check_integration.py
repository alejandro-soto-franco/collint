import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

import collint


def test_clean_figure_has_no_collisions():
    fig, ax = plt.subplots()
    ax.plot([0, 1, 2], [0, 1, 0])
    rep = collint.check(fig)
    assert not rep.has_collisions
    plt.close(fig)


def test_crowded_tick_labels_flag_text_text():
    fig, ax = plt.subplots(figsize=(2, 2))
    ax.plot(range(100), range(100))
    ax.set_xticks(range(0, 100))
    ax.set_xticklabels([f"label{i}" for i in range(100)])
    rep = collint.check(fig)
    kinds = {c.kind for c in rep.collisions}
    assert "text_text" in kinds
    plt.close(fig)


def test_legend_over_dense_data_flags_over_data():
    fig, ax = plt.subplots()
    x = np.linspace(0, 1, 500)
    for k in range(20):
        ax.plot(x, np.sin(x * (k + 1)))
    ax.legend([f"s{k}" for k in range(20)], loc="center")
    rep = collint.check(fig)
    kinds = {c.kind for c in rep.collisions}
    assert "over_data" in kinds
    plt.close(fig)


def test_legend_over_sparse_empty_region_does_not_flag_over_data():
    fig, ax = plt.subplots()
    # all data in the lower-left; legend forced to the empty upper-right
    ax.plot([0, 0.1, 0.2], [0, 0.05, 0.1], label="s")
    ax.set_xlim(0, 1)
    ax.set_ylim(0, 1)
    ax.legend(loc="upper right")
    rep = collint.check(fig)
    over = [c for c in rep.collisions if c.kind == "over_data"]
    assert over == []
    plt.close(fig)
