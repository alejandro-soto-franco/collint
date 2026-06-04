"""Walk a matplotlib Figure into tagged display-space boxes.

This is the only module that reads matplotlib geometry. It never mutates the
figure. All boxes are (x0, y0, x1, y1) in display pixels.
"""

from ._types import TaggedBox


def _bbox_to_tuple(bbox):
    return (float(bbox.x0), float(bbox.y0), float(bbox.x1), float(bbox.y1))


def extract_boxes(fig):
    """Return a list of TaggedBox for every relevant artist in `fig`.

    Requires the figure to have been drawn (a renderer must exist). Use
    collint.render.ensure_drawn first; check() does this for you.
    """
    renderer = fig.canvas.get_renderer()
    fig_box = _bbox_to_tuple(fig.bbox)
    boxes = [TaggedBox(fig_box, fig_box, "figure-region", "figure")]

    for ai, ax in enumerate(fig.axes):
        ax_box = _bbox_to_tuple(ax.bbox)
        axid = f"axes{ai}"
        boxes.append(TaggedBox(ax_box, fig_box, "axes-region", axid))

        # Title (clipped by the figure, it lives above the axes).
        title = ax.title
        if title.get_text():
            boxes.append(
                TaggedBox(_bbox_to_tuple(title.get_window_extent(renderer)), fig_box, "title", f"{axid}.title")
            )

        # Axis labels (clipped by the figure).
        for axis in (ax.xaxis, ax.yaxis):
            lbl = axis.label
            if lbl.get_text():
                boxes.append(
                    TaggedBox(
                        _bbox_to_tuple(lbl.get_window_extent(renderer)),
                        fig_box,
                        "axis-label",
                        f"{axid}.axislabel.{id(lbl)}",
                    )
                )

        # Tick labels (clipped by the figure; they sit just outside the axes).
        # Only labels whose tick lies within the axis view are actually drawn;
        # matplotlib clips off-view tick labels to the axes, so skip them.
        for axis in (ax.xaxis, ax.yaxis):
            vmin, vmax = axis.get_view_interval()
            lo, hi = (vmin, vmax) if vmin <= vmax else (vmax, vmin)
            locs = axis.get_majorticklocs()
            labels = axis.get_majorticklabels()
            for loc, tick_text in zip(locs, labels):
                if not tick_text.get_text():
                    continue
                if loc < lo or loc > hi:
                    continue
                boxes.append(
                    TaggedBox(
                        _bbox_to_tuple(tick_text.get_window_extent(renderer)),
                        fig_box,
                        "tick-label",
                        f"{axid}.tick.{id(tick_text)}",
                    )
                )

        # Legend (clipped by the figure).
        leg = ax.get_legend()
        if leg is not None:
            boxes.append(
                TaggedBox(_bbox_to_tuple(leg.get_window_extent(renderer)), fig_box, "legend", f"{axid}.legend")
            )

        # Data artists (clipped by the axes). Lines and collections.
        for li, line in enumerate(ax.lines):
            try:
                wb = line.get_window_extent(renderer)
            except Exception:
                continue
            if wb.width <= 0 or wb.height <= 0:
                continue
            boxes.append(TaggedBox(_bbox_to_tuple(wb), ax_box, "data-artist", f"{axid}.line{li}"))
        for ci, coll in enumerate(ax.collections):
            try:
                wb = coll.get_window_extent(renderer)
            except Exception:
                continue
            if wb.width <= 0 or wb.height <= 0:
                continue
            boxes.append(TaggedBox(_bbox_to_tuple(wb), ax_box, "data-artist", f"{axid}.coll{ci}"))

        # Annotations (clipped by the axes).
        for ti, txt in enumerate(ax.texts):
            if not txt.get_text():
                continue
            boxes.append(
                TaggedBox(_bbox_to_tuple(txt.get_window_extent(renderer)), ax_box, "annotation", f"{axid}.text{ti}")
            )

    # Suptitle (clipped by the figure).
    suptitle = fig._suptitle
    if suptitle is not None and suptitle.get_text():
        boxes.append(
            TaggedBox(_bbox_to_tuple(suptitle.get_window_extent(renderer)), fig_box, "suptitle", "suptitle")
        )

    return boxes
