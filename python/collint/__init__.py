"""collint: detect and auto-fix visual collisions in matplotlib figures."""

from . import _core  # type: ignore  # native pyo3 extension, no static stub
from .extract import extract_boxes
from .render import capture_rgba, ensure_drawn, hidden

__all__ = ["check", "autofix", "install", "extract_boxes"]


def _sample_bg(buf):
    """Estimate background RGB from the four buffer corners."""
    corners = [buf[0, 0], buf[0, -1], buf[-1, 0], buf[-1, -1]]
    rgb = [int(sum(int(c[ch]) for c in corners) / len(corners)) for ch in range(3)]
    return (rgb[0], rgb[1], rgb[2])


def _display_box_to_region(bx, height):
    """Convert a display-space box (origin bottom-left) to buffer rows/cols
    (origin top-left): (row0, col0, row1, col1)."""
    x0, y0, x1, y1 = bx
    row0 = int(max(0, height - y1))
    row1 = int(max(0, height - y0))
    col0 = int(max(0, x0))
    col1 = int(max(0, x1))
    return (row0, col0, row1, col1)


def check(fig, *, use_pixels=True, overlap_fraction=None, clip_px=None, ink_fraction=0.05):
    """Detect collisions in `fig`. Returns a CollisionReport. Never mutates `fig`."""
    ensure_drawn(fig)
    boxes = [b.as_tuple() for b in extract_boxes(fig)]
    collisions, candidates = _core.classify(boxes, overlap_fraction, clip_px)

    if candidates and use_pixels:
        confirmed = _confirm_over_data(fig, candidates, ink_fraction)
        collisions.extend(confirmed)
    elif candidates and not use_pixels:
        # geometry-only fallback: trust the geometric overlap
        for cand in candidates:
            collisions.append(
                _core.Collision(
                    "over_data",
                    cand.decoration_role,
                    cand.decoration_id,
                    cand.decoration_box,
                    "data-artist",
                    cand.data_id,
                    cand.data_box,
                    cand.overlap,
                )
            )

    return _core.CollisionReport(collisions)


def _artist_by_id(fig, artist_id):
    """Resolve a decoration id from extract.py back to its artist."""
    # ids look like "axes0.legend" or "axes0.text3"
    for ai, ax in enumerate(fig.axes):
        prefix = f"axes{ai}."
        if not artist_id.startswith(prefix):
            continue
        rest = artist_id[len(prefix):]
        if rest == "legend":
            return ax.get_legend()
        if rest.startswith("text"):
            return ax.texts[int(rest[len("text"):])]
    return None


def _confirm_over_data(fig, candidates, ink_fraction):
    """Hide all candidate decorations, redraw once, and sample ink under each."""
    decorations = []
    seen = set()
    for cand in candidates:
        if cand.decoration_id in seen:
            continue
        seen.add(cand.decoration_id)
        art = _artist_by_id(fig, cand.decoration_id)
        if art is not None:
            decorations.append(art)

    confirmed = []
    if not decorations:
        return confirmed

    base = capture_rgba(fig)
    height = base.shape[0]
    bg = _sample_bg(base)

    with hidden(decorations):
        fig.canvas.draw()
        buf = capture_rgba(fig)
        for cand in candidates:
            region = _display_box_to_region(cand.decoration_box, height)
            coverage = _core.ink_coverage(buf, region, bg, 8)
            if coverage >= ink_fraction:
                confirmed.append(
                    _core.Collision(
                        "over_data",
                        cand.decoration_role,
                        cand.decoration_id,
                        cand.decoration_box,
                        "data-artist",
                        cand.data_id,
                        cand.data_box,
                        cand.overlap,
                    )
                )
    # restore the drawn state
    fig.canvas.draw()
    return confirmed


def autofix(fig, **kwargs):
    from .autofix import autofix as _autofix

    return _autofix(fig, **kwargs)


def install():
    from .hook import install as _install

    return _install()
