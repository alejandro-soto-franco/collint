"""Auto-fix by orchestrating matplotlib's own layout machinery.

Greedy ladder: apply a strategy, re-detect, keep it only if total severity did
not increase. v1 is orchestration-only: no custom solver.
"""

from dataclasses import dataclass

from . import check


@dataclass
class AutofixResult:
    actions: list
    residual: object  # CollisionReport


def _strategy_tight_layout(fig):
    try:
        fig.tight_layout()
        return True
    except Exception:
        return False


def _strategy_constrained_layout(fig):
    try:
        fig.set_layout_engine("constrained")
        fig.canvas.draw()
        return True
    except Exception:
        return False


def _strategy_relocate_legends(fig):
    moved = False
    for ax in fig.axes:
        leg = ax.get_legend()
        if leg is None:
            continue
        handles, labels = ax.get_legend_handles_labels()
        if handles:
            ax.legend(handles, labels, loc="best")
            moved = True
    return moved


def _strategy_rotate_xticks(fig):
    rotated = False
    for ax in fig.axes:
        if ax.get_xticklabels():
            for lbl in ax.get_xticklabels():
                lbl.set_rotation(45)
                lbl.set_ha("right")
            rotated = True
    return rotated


STRATEGIES = [
    ("constrained_layout", _strategy_constrained_layout),
    ("tight_layout", _strategy_tight_layout),
    ("relocate_legends", _strategy_relocate_legends),
    ("rotate_xticks", _strategy_rotate_xticks),
]


def autofix(fig, **check_kwargs):
    """Apply remedies greedily. Returns AutofixResult(actions, residual report)."""
    actions = []
    best = check(fig, **check_kwargs)

    for name, strategy in STRATEGIES:
        if not best.has_collisions:
            break
        before = best.total()
        applied = strategy(fig)
        if not applied:
            continue
        fig.canvas.draw()
        after_report = check(fig, **check_kwargs)
        if after_report.total() <= before:
            best = after_report
            actions.append(name)
        # if a strategy made it worse we cannot trivially revert layout calls,
        # so we keep the better-scoring report we already had and stop applying
        # further escalations that build on a regression
        else:
            break

    return AutofixResult(actions=actions, residual=best)
