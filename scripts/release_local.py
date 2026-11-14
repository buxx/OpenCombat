#!/usr/bin/env python3
# WARNING: AI generated script
"""Run a GitHub Actions workflow (default: .github/workflows/release.yml) locally.

It reads the workflow, orders jobs by `needs`, expands the matrix (keeping only
entries matching the local OS), evaluates `${{ ... }}` expressions and simple
`if:` conditions, and executes every `run:` step with bash, like a runner would.

`uses:` steps are emulated:
  - actions/checkout        -> current repo (or a clean `git worktree` with --clean-checkout)
  - dtolnay/rust-toolchain  -> checks that cargo is installed
  - Swatinem/rust-cache     -> skipped
  - actions/upload-artifact -> copied to .release-local/artifacts/<name>/
  - actions/download-artifact -> copied from .release-local/artifacts/
  - anything else           -> skipped with a warning

Safety defaults (override with flags):
  - steps using `sudo` are skipped unless --allow-sudo
  - steps calling `gh release` are only printed unless --publish

Usage:
  scripts/release_local.py                     # build + package + fake release
  scripts/release_local.py --ref-name v0.1.0   # simulate the tag name
  scripts/release_local.py --job build         # run only one job
  scripts/release_local.py --clean-checkout    # build from committed files only
  scripts/release_local.py --list              # show what would run
"""

from __future__ import annotations

import argparse
import os
import platform
import re
import shutil
import subprocess
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    sys.exit("PyYAML is required: pip install pyyaml (or apt install python3-yaml)")

REPO = Path(__file__).resolve().parent.parent
LOCAL_DIR = REPO / ".release-local"
ARTIFACTS_DIR = LOCAL_DIR / "artifacts"

EXPR_RE = re.compile(r"\$\{\{\s*(.*?)\s*\}\}")


def color(text: str, code: str) -> str:
    return f"\033[{code}m{text}\033[0m" if sys.stdout.isatty() else text


def info(msg: str) -> None:
    print(color(msg, "1;34"), flush=True)


def warn(msg: str) -> None:
    print(color(msg, "1;33"), flush=True)


def ok(msg: str) -> None:
    print(color(msg, "1;32"), flush=True)


def local_runner_os() -> str:
    return {"Linux": "Linux", "Windows": "Windows", "Darwin": "macOS"}[platform.system()]


def runs_on_to_os(runs_on: str) -> str:
    runs_on = runs_on.lower()
    if runs_on.startswith("windows"):
        return "Windows"
    if runs_on.startswith("macos"):
        return "macOS"
    return "Linux"


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=REPO, text=True).strip()


def default_ref_name() -> str:
    try:
        return git("describe", "--tags", "--always", "--dirty")
    except subprocess.CalledProcessError:
        return "local"


def default_repository() -> str:
    try:
        url = git("remote", "get-url", "origin")
    except subprocess.CalledProcessError:
        return "local/local"
    m = re.search(r"[:/]([^/:]+/[^/]+?)(?:\.git)?$", url)
    return m.group(1) if m else "local/local"


# --------------------------------------------------------------------------- #
# Expressions
# --------------------------------------------------------------------------- #


class Context:
    def __init__(self, github: dict, runner: dict, matrix: dict, env: dict):
        self.data = {"github": github, "runner": runner, "matrix": matrix, "env": env}

    def lookup(self, path: str) -> str:
        node: object = self.data
        for part in path.split("."):
            if isinstance(node, dict) and part in node:
                node = node[part]
            else:
                return ""
        return "" if node is None else str(node)

    def substitute(self, text: str) -> str:
        return EXPR_RE.sub(lambda m: self.evaluate_value(m.group(1)), text)

    def evaluate_value(self, expr: str) -> str:
        expr = expr.strip()
        if re.fullmatch(r"[A-Za-z_][\w.-]*", expr):
            return self.lookup(expr)
        return str(self.evaluate_bool(expr)).lower()

    def evaluate_bool(self, expr: str) -> bool:
        """Tiny evaluator for `if:` expressions (==, !=, &&, ||, !, literals)."""
        m = EXPR_RE.fullmatch(expr.strip())
        if m:
            expr = m.group(1)
        if expr.strip() in ("success()", "always()"):
            return True
        if expr.strip() in ("failure()", "cancelled()"):
            return False

        def repl_path(match: re.Match) -> str:
            return repr(self.lookup(match.group(0)))

        py = re.sub(r"'([^']*)'", lambda mm: repr(mm.group(1)), expr)
        py = re.sub(r"(?<!['\w])(github|runner|matrix|env)(\.[\w-]+)+", repl_path, py)
        py = py.replace("&&", " and ").replace("||", " or ")
        py = re.sub(r"!(?!=)", " not ", py)
        py = re.sub(r"\btrue\b", "True", py)
        py = re.sub(r"\bfalse\b", "False", py)
        try:
            return bool(eval(py, {"__builtins__": {}}, {}))  # noqa: S307 - local, trusted workflow
        except Exception as exc:  # pragma: no cover
            warn(f"  ! cannot evaluate `if: {expr}` ({exc}); assuming true")
            return True


# --------------------------------------------------------------------------- #
# Runner
# --------------------------------------------------------------------------- #


class Runner:
    def __init__(self, args: argparse.Namespace):
        self.args = args
        self.workflow = yaml.safe_load(Path(args.workflow).read_text())
        self.local_os = local_runner_os()
        self.github = {
            "ref_name": args.ref_name,
            "ref": f"refs/tags/{args.ref_name}",
            "repository": args.repository,
            "token": os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN", ""),
            "workspace": str(REPO),
            "sha": git("rev-parse", "HEAD"),
            "event_name": "push",
        }
        self.worktree: Path | None = None

    # -- job ordering ------------------------------------------------------- #

    def ordered_jobs(self) -> list[str]:
        jobs = self.workflow["jobs"]
        done: list[str] = []

        def visit(name: str, stack: tuple = ()) -> None:
            if name in done:
                return
            if name in stack:
                sys.exit(f"cycle in job needs: {' -> '.join(stack + (name,))}")
            needs = jobs[name].get("needs", [])
            for need in [needs] if isinstance(needs, str) else needs:
                visit(need, stack + (name,))
            done.append(name)

        for name in jobs:
            visit(name)
        if self.args.job:
            unknown = set(self.args.job) - set(jobs)
            if unknown:
                sys.exit(f"unknown job(s): {', '.join(unknown)}")
            done = [j for j in done if j in self.args.job]
        return done

    def matrix_entries(self, job: dict) -> list[dict]:
        matrix = (job.get("strategy") or {}).get("matrix")
        if not matrix:
            return [{}]
        entries: list[dict] = [{}]
        for key, values in matrix.items():
            if key in ("include", "exclude"):
                continue
            entries = [{**e, key: v} for e in entries for v in values]
        entries += matrix.get("include", [])
        for ex in matrix.get("exclude", []):
            entries = [e for e in entries if not all(e.get(k) == v for k, v in ex.items())]
        return entries

    # -- main --------------------------------------------------------------- #

    def run(self) -> int:
        info(f"Workflow : {self.args.workflow}")
        info(f"ref_name : {self.github['ref_name']}   repository: {self.github['repository']}")
        info(f"local OS : {self.local_os}")
        if not self.args.list:
            if not self.args.job:  # keep previous artifacts when running a single job
                shutil.rmtree(ARTIFACTS_DIR, ignore_errors=True)
            ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)
        try:
            for job_name in self.ordered_jobs():
                job = self.workflow["jobs"][job_name]
                for matrix in self.matrix_entries(job):
                    if not self.run_job(job_name, job, matrix):
                        return 1
        finally:
            self.cleanup_worktree()
        ok("\nWorkflow completed successfully.")
        if not self.args.list:
            print(f"Artifacts: {ARTIFACTS_DIR}")
        return 0

    def run_job(self, job_name: str, job: dict, matrix: dict) -> bool:
        pre_ctx = Context(self.github, {"os": self.local_os}, matrix, {})
        runs_on = pre_ctx.substitute(str(job.get("runs-on", "ubuntu-latest")))
        job_os = runs_on_to_os(runs_on)
        label = f"{job_name}" + (f" ({', '.join(map(str, matrix.values()))})" if matrix else "")
        print()
        if job_os != self.local_os:
            warn(f"=== Job {label}: skipped (runs-on {runs_on}, local OS is {self.local_os})")
            return True
        info(f"=== Job {label} [runs-on {runs_on}]")

        has_checkout = any(str(s.get("uses", "")).startswith("actions/checkout") for s in job["steps"])
        if has_checkout:
            workdir = self.checkout_dir()
        else:
            # Fresh, empty workspace like a new runner.
            workdir = LOCAL_DIR / "jobs" / job_name
            if not self.args.list:
                shutil.rmtree(workdir, ignore_errors=True)
                workdir.mkdir(parents=True)

        github_env_file = LOCAL_DIR / f"github_env_{job_name}"
        if not self.args.list:
            github_env_file.write_text("")
        job_env: dict[str, str] = {}
        for k, v in (job.get("env") or {}).items():
            job_env[k] = Context(self.github, {"os": job_os}, matrix, job_env).substitute(str(v))

        for idx, step in enumerate(job["steps"], 1):
            ctx = Context(self.github, {"os": job_os}, matrix, job_env)
            name = step.get("name") or step.get("uses") or (step.get("run", "").splitlines() or ["?"])[0]
            name = ctx.substitute(str(name))
            prefix = f"--- [{idx}/{len(job['steps'])}] {name}"

            if "if" in step and not ctx.evaluate_bool(str(step["if"])):
                warn(f"{prefix}: skipped (if: {step['if']} is false)")
                continue

            if self.args.list:
                print(prefix)
                continue

            info(prefix)
            if "uses" in step:
                success = self.run_uses(step, ctx, workdir)
            else:
                success = self.run_shell(step, ctx, workdir, job_os, job_env, github_env_file)
            if not success:
                print(color(f"Step failed: {name}", "1;31"))
                return False

            job_env.update(self.read_github_env(github_env_file))
        return True

    # -- checkout ------------------------------------------------------------ #

    def checkout_dir(self) -> Path:
        if not self.args.clean_checkout:
            return REPO
        if self.worktree is None and not self.args.list:
            self.worktree = LOCAL_DIR / "checkout"
            if self.worktree.exists():
                subprocess.run(["git", "worktree", "remove", "--force", str(self.worktree)], cwd=REPO)
                shutil.rmtree(self.worktree, ignore_errors=True)
            subprocess.check_call(
                ["git", "worktree", "add", "--detach", str(self.worktree), "HEAD"], cwd=REPO
            )
        return self.worktree or REPO

    def cleanup_worktree(self) -> None:
        if self.worktree and not self.args.keep_checkout:
            subprocess.run(
                ["git", "worktree", "remove", "--force", str(self.worktree)],
                cwd=REPO,
                stdout=subprocess.DEVNULL,
            )

    # -- uses: emulation ----------------------------------------------------- #

    def run_uses(self, step: dict, ctx: Context, workdir: Path) -> bool:
        uses = step["uses"]
        action = uses.split("@")[0]
        with_ = {k: ctx.substitute(str(v)) for k, v in (step.get("with") or {}).items()}

        if action == "actions/checkout":
            print(f"    (emulated) workspace: {workdir}")
            return True
        if action == "dtolnay/rust-toolchain":
            found = shutil.which("cargo")
            print(f"    (emulated) cargo: {found or 'NOT FOUND'}")
            if found:
                subprocess.run(["cargo", "--version"])
            return bool(found)
        if action == "Swatinem/rust-cache":
            print("    (emulated) skipped, local target/ is the cache")
            return True
        if action == "actions/upload-artifact":
            name = with_.get("name") or "artifact"
            dest = ARTIFACTS_DIR / name
            dest.mkdir(parents=True, exist_ok=True)
            paths = [p for p in with_.get("path", "").splitlines() if p.strip()]
            if not paths:
                print("    path is empty (did the previous step write to $GITHUB_ENV?)")
                return False
            for p in paths:
                src = workdir / p.strip()
                if not src.exists():
                    print(f"    missing file: {src}")
                    return False
                (shutil.copytree if src.is_dir() else shutil.copy2)(src, dest / src.name)
                print(f"    uploaded {src.name} -> {dest}")
            return True
        if action == "actions/download-artifact":
            target = workdir / with_.get("path", ".")
            target.mkdir(parents=True, exist_ok=True)
            merge = with_.get("merge-multiple", "false").lower() == "true"
            only = with_.get("name")
            for art in sorted(ARTIFACTS_DIR.iterdir()):
                if only and art.name != only:
                    continue
                dest = target if (merge or only) else target / art.name
                shutil.copytree(art, dest, dirs_exist_ok=True)
                print(f"    downloaded {art.name} -> {dest}")
            return True

        warn(f"    unsupported action `{uses}`: skipped")
        return True

    # -- run: steps ---------------------------------------------------------- #

    def run_shell(
        self,
        step: dict,
        ctx: Context,
        workdir: Path,
        job_os: str,
        job_env: dict,
        github_env_file: Path,
    ) -> bool:
        script = ctx.substitute(step["run"])

        if re.search(r"\bsudo\b", script) and not self.args.allow_sudo:
            warn("    uses sudo: skipped (pass --allow-sudo to run it). Script:")
            print("    " + script.strip().replace("\n", "\n    "))
            return True
        if re.search(r"\bgh\s+release\b", script) and not self.args.publish:
            warn("    publishes a GitHub release: dry-run (pass --publish to run it). Would run:")
            print("    " + script.strip().replace("\n", "\n    "))
            target = workdir / "dist"
            if target.exists():
                print("    with files:")
                for f in sorted(target.iterdir()):
                    print(f"      {f.name}  ({f.stat().st_size / 1e6:.1f} MB)")
            return True

        env = {
            **os.environ,
            **job_env,
            "CI": "true",
            "GITHUB_ACTIONS": "true",
            "GITHUB_ENV": str(github_env_file),
            "GITHUB_WORKSPACE": str(workdir),
            "GITHUB_REF_NAME": self.github["ref_name"],
            "GITHUB_REF": self.github["ref"],
            "GITHUB_REPOSITORY": self.github["repository"],
            "GITHUB_SHA": self.github["sha"],
            "RUNNER_OS": job_os,
        }
        for k, v in (step.get("env") or {}).items():
            env[k] = ctx.substitute(str(v))

        shell = step.get("shell")
        if shell in (None, "bash"):
            # Same as GitHub: default is `bash -e`, explicit `shell: bash` adds pipefail.
            cmd = ["bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", script]
            if shell is None:
                cmd = ["bash", "-e", "-c", script]
        elif shell == "sh":
            cmd = ["sh", "-e", "-c", script]
        else:
            cmd = [*shell.replace("{0}", "").split(), script]

        cwd = workdir / step["working-directory"] if "working-directory" in step else workdir
        if self.args.verbose:
            print("    " + script.strip().replace("\n", "\n    "))
        return subprocess.run(cmd, cwd=cwd, env=env).returncode == 0

    @staticmethod
    def read_github_env(path: Path) -> dict[str, str]:
        result: dict[str, str] = {}
        if not path.exists():
            return result
        lines = path.read_text().splitlines()
        i = 0
        while i < len(lines):
            line = lines[i]
            if "<<" in line and "=" not in line.split("<<")[0]:
                key, delim = line.split("<<", 1)
                value = []
                i += 1
                while i < len(lines) and lines[i] != delim:
                    value.append(lines[i])
                    i += 1
                result[key] = "\n".join(value)
            elif "=" in line:
                key, value = line.split("=", 1)
                result[key] = value
            i += 1
        return result


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--workflow", default=str(REPO / ".github/workflows/release.yml"))
    p.add_argument("--ref-name", default=None, help="simulated tag name (default: git describe)")
    p.add_argument("--repository", default=None, help="owner/repo (default: from origin remote)")
    p.add_argument("--job", action="append", help="run only this job (repeatable)")
    p.add_argument("--list", action="store_true", help="only list jobs/steps that would run")
    p.add_argument("--clean-checkout", action="store_true", help="build in a clean git worktree of HEAD")
    p.add_argument("--keep-checkout", action="store_true", help="keep the clean worktree afterwards")
    p.add_argument("--allow-sudo", action="store_true", help="run steps that use sudo")
    p.add_argument("--publish", action="store_true", help="really run `gh release ...`")
    p.add_argument("-v", "--verbose", action="store_true", help="print scripts before running")
    args = p.parse_args()
    args.ref_name = args.ref_name or default_ref_name()
    args.repository = args.repository or default_repository()
    return Runner(args).run()


if __name__ == "__main__":
    sys.exit(main())
