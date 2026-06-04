"""Opt-in savefig hook: warn when a figure is saved with collisions."""

import warnings

from matplotlib.figure import Figure

import collint

_ORIGINAL = None


def install():
    """Monkeypatch Figure.savefig to run collint.check and warn on collisions."""
    global _ORIGINAL
    if _ORIGINAL is not None:
        return
    _ORIGINAL = Figure.savefig

    def savefig(self, *args, **kwargs):
        result = _ORIGINAL(self, *args, **kwargs)
        try:
            report = collint.check(self)
            if report.has_collisions:
                warnings.warn(
                    f"collint: {len(report.collisions)} collisions in saved figure",
                    stacklevel=2,
                )
        except Exception:
            pass
        return result

    Figure.savefig = savefig


def uninstall():
    global _ORIGINAL
    if _ORIGINAL is not None:
        Figure.savefig = _ORIGINAL
        _ORIGINAL = None
