"""Tests for scripts/local-ci.sh.

The script exists because a hand-copied list of gate commands drifts from the
workflow, and the drift is silent: it shows up as a green local gate followed by
a red CI. So the property worth testing is not that it runs — it is that the
list it runs is **derived** from the workflow rather than baked in.
"""

from __future__ import annotations

import os
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPT = REPO_ROOT / "scripts" / "local-ci.sh"


def run(workflow: Path | None, *args: str) -> subprocess.CompletedProcess:
    env = dict(os.environ)
    if workflow is not None:
        env["WORKFLOW"] = str(workflow)
    return subprocess.run(
        [str(SCRIPT), *args], capture_output=True, text=True, cwd=REPO_ROOT, env=env
    )


def workflow_with(step_name: str, run_line: str, extra: str = "") -> str:
    return textwrap.dedent(f"""\
        name: synthetic
        jobs:
          lint:
            steps:
              - name: {step_name}
                run: {run_line}
        {extra}""")


class DerivationTests(unittest.TestCase):
    """A gate added to the workflow appears without touching the script."""

    def test_a_gate_only_the_workflow_knows_about_is_listed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            wf = Path(tmp) / "ci.yml"
            wf.write_text(workflow_with("Gate nobody hard-coded", "true"), encoding="utf-8")
            result = run(wf, "--list")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("Gate nobody hard-coded", result.stdout)

    def test_a_runner_only_step_is_reported_rather_than_omitted(self) -> None:
        # A gate skipped in silence is worth less than a gate that is absent:
        # the operator believes it ran.
        with tempfile.TemporaryDirectory() as tmp:
            wf = Path(tmp) / "ci.yml"
            wf.write_text(workflow_with("Needs the runner", "echo $GITHUB_SHA"), encoding="utf-8")
            result = run(wf, "--list")
            self.assertIn("SAUTÉE", result.stdout)
            self.assertIn("Needs the runner", result.stdout)


class RefusalTests(unittest.TestCase):
    """It fails loudly rather than reporting a pass it never established."""

    def test_a_failing_gate_makes_the_script_fail(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            wf = Path(tmp) / "ci.yml"
            wf.write_text(workflow_with("Gate that fails", "false"), encoding="utf-8")
            result = run(wf)
            self.assertEqual(result.returncode, 1)
            self.assertIn("Gate that fails", result.stderr)

    def test_a_missing_workflow_answers_2_not_1(self) -> None:
        # 2 is "cannot read", 1 is "refused". A tree this cannot read must not
        # wear the exit code of a refusal it never made.
        with tempfile.TemporaryDirectory() as tmp:
            result = run(Path(tmp) / "absent.yml")
            self.assertEqual(result.returncode, 2)

    def test_an_unknown_job_answers_2(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            wf = Path(tmp) / "ci.yml"
            wf.write_text(workflow_with("x", "true"), encoding="utf-8")
            env = dict(os.environ, WORKFLOW=str(wf), JOBS="does-not-exist")
            result = subprocess.run(
                [str(SCRIPT), "--list"], capture_output=True, text=True,
                cwd=REPO_ROOT, env=env,
            )
            self.assertEqual(result.returncode, 2)


class EmptyRunTests(unittest.TestCase):
    """Zero steps executed must never look like a pass."""

    def test_a_workflow_with_no_replayable_steps_answers_2(self) -> None:
        # Found by this very suite: an unexpected value crashed the reader, the
        # loop processed zero steps, and the script reported "all gates pass".
        # An operator would have believed the gates ran. Zero is now an error.
        with tempfile.TemporaryDirectory() as tmp:
            wf = Path(tmp) / "ci.yml"
            wf.write_text(
                "name: synthetic\njobs:\n  lint:\n    steps:\n      - uses: actions/checkout@v4\n",
                encoding="utf-8",
            )
            result = run(wf)
            self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
            self.assertIn("aucune étape", result.stderr)

    def test_a_boolean_run_value_does_not_silently_empty_the_run(self) -> None:
        # `run: true` is parsed by YAML as a boolean, not the string "true".
        with tempfile.TemporaryDirectory() as tmp:
            wf = Path(tmp) / "ci.yml"
            wf.write_text(workflow_with("Boolean run", "true"), encoding="utf-8")
            result = run(wf)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn("Portes rejouées: 1", result.stdout)


class EscapingTests(unittest.TestCase):
    """The bug this script shipped with, pinned so it cannot return."""

    def test_a_line_continuation_survives_the_round_trip(self) -> None:
        # Encoding the command through escape sequences turned a trailing `\`
        # into a literal `\` + `n`, and the shell received an argument `n`.
        # That produced a FALSE RED on clippy — the surest way to get a gate
        # switched off. The command is base64-carried now.
        with tempfile.TemporaryDirectory() as tmp:
            wf = Path(tmp) / "ci.yml"
            wf.write_text(textwrap.dedent("""\
                name: synthetic
                jobs:
                  lint:
                    steps:
                      - name: Continued command
                        run: |
                          test 1 -eq 1 \\
                            -a 2 -eq 2
                """), encoding="utf-8")
            result = run(wf)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
