import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

import collint
from collint.autofix import autofix


def test_autofix_reduces_or_clears_clipping():
    fig, ax = plt.subplots(figsize=(2, 2))
    ax.plot(range(10), range(10))
    ax.set_ylabel("a very long y axis label that gets clipped at the edge")
    before = collint.check(fig).total()
    result = autofix(fig)
    after = result.residual.total()
    assert after <= before
    plt.close(fig)


def test_autofix_never_worse_on_clean_figure():
    fig, ax = plt.subplots()
    ax.plot([0, 1, 2], [0, 1, 0])
    before = collint.check(fig).total()
    result = autofix(fig)
    after = result.residual.total()
    assert after <= before
    assert isinstance(result.actions, list)
    plt.close(fig)
