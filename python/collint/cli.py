"""Command-line entry point. Loads a Python file that defines a module-level
`fig`, checks it, prints JSON, optionally writes an overlay, exits non-zero on
collisions."""

import argparse
import importlib.util
import sys

import collint
from collint.overlay import render_overlay


def _load_fig(path):
    spec = importlib.util.spec_from_file_location("collint_target", path)
    if spec is None or spec.loader is None:
        print(f"error: cannot load {path}", file=sys.stderr)
        sys.exit(2)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    fig = getattr(module, "fig", None)
    if fig is None:
        print(f"error: {path} defines no module-level `fig`", file=sys.stderr)
        sys.exit(2)
    return fig


def main(argv=None):
    parser = argparse.ArgumentParser(prog="collint")
    parser.add_argument("script", help="Python file defining a module-level `fig`")
    parser.add_argument("--overlay", help="write an annotated overlay PNG to this path")
    parser.add_argument("--no-pixels", action="store_true", help="disable pixel confirmation")
    args = parser.parse_args(argv)

    fig = _load_fig(args.script)
    report = collint.check(fig, use_pixels=not args.no_pixels)
    print(report.to_json())
    if args.overlay:
        render_overlay(fig, report, args.overlay)
    sys.exit(report.exit_code())


if __name__ == "__main__":
    main()
