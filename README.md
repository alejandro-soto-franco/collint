# collint

Detect and auto-fix visual collisions in matplotlib figures.

`collint` inspects a rendered figure and finds overlapping or clipped elements:

- text-text: tick labels, annotations, axis labels, titles, or legend text overlapping each other
- clipping: titles, axis labels, or tick labels cut off by the figure edge
- over-data: a legend or annotation covering plotted data
- element-element: overlapping subplots, colorbars, or a suptitle colliding with subplot titles

Detection is geometry-based over the rendered bounding boxes. The over-data case is confirmed by sampling the pixels underneath the decoration. A legend placed over an empty region of a sparse plot is reported only when data lies underneath.

## Install

Building requires a Rust toolchain (the core is a compiled extension).

```bash
pip install git+https://github.com/alejandro-soto-franco/collint
```

Or from a clone:

```bash
git clone https://github.com/alejandro-soto-franco/collint
cd collint
pip install .
```

Supported Python versions: 3.10 to 3.13.

## Quick start

```python
import matplotlib.pyplot as plt
import collint

fig, ax = plt.subplots(figsize=(2, 2))
ax.plot(range(100), range(100))
ax.set_xticks(range(0, 100))
ax.set_xticklabels([f"label{i}" for i in range(100)])

report = collint.check(fig)
print(report.has_collisions)        # True
print(report.severity)              # worst overlap fraction
print(report.to_json())             # machine-readable list
```

`check` never mutates the figure. It draws the figure on an Agg renderer if needed and works headless.

## Report

`check` returns a `CollisionReport`:

```python
report = collint.check(fig)

report.has_collisions      # bool
report.collisions          # list of Collision
report.severity            # max severity across collisions (0.0 if none)
report.total()             # summed severity
report.exit_code()         # 0 if clean, 1 if any collision
report.by_type("text_text")  # collisions of one kind
report.to_json()           # JSON string
```

Each `Collision` has the two elements involved:

```python
for c in report.collisions:
    print(c.kind)           # "text_text" | "clipping" | "over_data" | "element_element"
    print(c.a_role, c.a_id) # e.g. "tick-label", "axes0.tick.140..."
    print(c.a_box)          # (x0, y0, x1, y1) in display pixels
    print(c.b_role, c.b_id)
    print(c.b_box)
    print(c.severity)       # overlap fraction, or clip overhang in pixels
```

## Auto-fixing

`autofix` tries to resolve collisions by applying matplotlib's own layout tools (constrained layout, tight layout, legend relocation, tick rotation), re-checking after each step and keeping a change only if it does not make things worse.

```python
result = collint.autofix(fig)
print(result.actions)            # e.g. ["constrained_layout"]
print(result.residual.has_collisions)  # what remains after fixing
```

`autofix` reports exactly which steps it applied. It never returns a figure worse than the one it received.

## Annotated overlay

Save a copy of the figure with every colliding region outlined in red:

```python
from collint.overlay import render_overlay

report = collint.check(fig)
render_overlay(fig, report, "overlay.png")
```

The overlay is written to disk and the original figure is left unchanged.

## Command line

Point the CLI at a Python file that defines a module-level `fig`. It prints the JSON report and exits non-zero when collisions are found. A figure-generation pipeline can fail on that exit status.

```bash
collint myfigure.py
collint myfigure.py --overlay overlay.png
collint myfigure.py --no-pixels      # skip pixel confirmation, geometry only
```

```python
# myfigure.py
import matplotlib.pyplot as plt

fig, ax = plt.subplots()
ax.plot([0, 1, 2], [0, 1, 0])
```

Exit code is 0 when the figure is clean and 1 when it has collisions.

## pytest integration

Installing `collint` registers an `assert_no_collisions` fixture:

```python
def test_my_plot_is_clean(assert_no_collisions):
    fig = build_my_plot()
    assert_no_collisions(fig)
```

The assertion fails with the JSON report when the figure has collisions.

## savefig hook

Opt in to a warning every time a figure with collisions is saved:

```python
import collint
collint.install()

fig.savefig("out.png")   # warns if out.png has visual collisions
```

## Configuration

Thresholds can be passed to `check` (and through `autofix`):

```python
collint.check(
    fig,
    use_pixels=True,        # confirm over-data collisions by sampling pixels
    overlap_fraction=0.02,  # minimum overlap (fraction of the smaller box)
    clip_px=1.0,            # minimum clip overhang in pixels
    ink_fraction=0.05,      # minimum data ink under a decoration to confirm over-data
)
```

Defaults ignore sub-pixel and antialiasing noise. Set `use_pixels=False` to run geometry-only detection. This is faster but reports any legend or annotation whose bounding box overlaps a data artist, even over empty regions.
