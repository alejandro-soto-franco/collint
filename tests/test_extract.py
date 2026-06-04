import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

from collint.extract import extract_boxes


def test_extract_returns_roles_for_a_basic_figure():
    fig, ax = plt.subplots()
    ax.plot([0, 1, 2], [0, 1, 4], label="line")
    ax.set_title("title")
    ax.set_xlabel("x")
    ax.legend()
    fig.canvas.draw()
    boxes = extract_boxes(fig)
    roles = {b.role for b in boxes}
    assert "title" in roles
    assert "axis-label" in roles
    assert "legend" in roles
    assert "data-artist" in roles
    assert "axes-region" in roles
    assert "figure-region" in roles
    # every box is a 4-tuple of finite floats
    for b in boxes:
        assert len(b.bx) == 4
        assert all(isinstance(v, float) for v in b.bx)
    plt.close(fig)
