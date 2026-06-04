import subprocess
import sys
import textwrap


def test_cli_exits_nonzero_on_collisions(tmp_path):
    script = tmp_path / "figmod.py"
    script.write_text(
        textwrap.dedent(
            """
            import matplotlib
            matplotlib.use("Agg")
            import matplotlib.pyplot as plt
            fig, ax = plt.subplots(figsize=(2, 2))
            ax.plot(range(50), range(50))
            ax.set_xticks(range(0, 50))
            ax.set_xticklabels([f"label{i}" for i in range(50)])
            """
        )
    )
    proc = subprocess.run(
        [sys.executable, "-m", "collint.cli", str(script)],
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 1
    assert "text_text" in proc.stdout


def test_cli_exits_zero_on_clean(tmp_path):
    script = tmp_path / "clean.py"
    script.write_text(
        textwrap.dedent(
            """
            import matplotlib
            matplotlib.use("Agg")
            import matplotlib.pyplot as plt
            fig, ax = plt.subplots()
            ax.plot([0, 1, 2], [0, 1, 0])
            """
        )
    )
    proc = subprocess.run(
        [sys.executable, "-m", "collint.cli", str(script)],
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0
