"""Render a copy of the figure with colliding boxes outlined in red."""

import matplotlib.patches as mpatches
from matplotlib.transforms import IdentityTransform


def render_overlay(fig, report, path):
    """Draw red rectangles over every colliding box and save to `path`.

    Boxes are added as figure-level patches in display coords, then removed so
    the user's figure is left unchanged.
    """
    added = []
    for c in report.collisions:
        for bx in (c.a_box, c.b_box):
            x0, y0, x1, y1 = bx
            rect = mpatches.Rectangle(
                (x0, y0),
                x1 - x0,
                y1 - y0,
                transform=IdentityTransform(),  # display (pixel) coords
                fill=False,
                edgecolor="red",
                linewidth=1.5,
                zorder=1000,
            )
            fig.patches.append(rect)
            added.append(rect)
    try:
        fig.savefig(path, dpi=fig.dpi)
    finally:
        for rect in added:
            fig.patches.remove(rect)
