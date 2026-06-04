"""Renderer access, RGBA buffer capture, and reversible state mutation.

The only module that mutates a Figure. Every mutation is wrapped in a context
manager that restores the prior state, even when the body raises.
"""

from contextlib import contextmanager

import numpy as np
from matplotlib.backends.backend_agg import FigureCanvasAgg


def ensure_drawn(fig):
    """Guarantee the figure has an Agg renderer and is drawn.

    Attaches an Agg canvas if the current backend has no usable renderer.
    """
    canvas = fig.canvas
    if not isinstance(canvas, FigureCanvasAgg):
        FigureCanvasAgg(fig)  # rebinds fig.canvas
    fig.canvas.draw()
    return fig.canvas.get_renderer()


def capture_rgba(fig):
    """Return the current Agg buffer as an (H, W, 4) uint8 array (row 0 = top)."""
    buf = np.asarray(fig.canvas.buffer_rgba())
    return buf


@contextmanager
def hidden(artists):
    """Temporarily set each artist invisible, restoring prior visibility on exit."""
    prior = [(a, a.get_visible()) for a in artists]
    try:
        for a, _ in prior:
            a.set_visible(False)
        yield
    finally:
        for a, vis in prior:
            a.set_visible(vis)
