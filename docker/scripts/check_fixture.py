"""Assert that `scuv migrate list --json` found an image's fixture env, and
that a dry run of migrating it reads its packages.

Usage: scuv migrate list --json | python3 check_fixture.py <name> <source_type> <python> <scuv>

<python> is the major.minor the fixture was created with, e.g. 3.12.
<scuv> is the binary to run the dry run with.

The migration tests build their own fake trees, and `migrate list` exits 0
when it finds nothing, so a broken or missing fixture in the Docker image
would otherwise leave the integration job green. This checks that the named
env was discovered from the expected tool, is ready to migrate, is listed
with the expected Python, and that its interpreter runs and is that Python.
"""

import json
import subprocess
import sys
from pathlib import Path


def main() -> int:
    name, source_type, expected, scuv = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
    response = json.load(sys.stdin)
    envs = response["data"]["environments"]
    matches = [e for e in envs if e["name"] == name]
    if len(matches) != 1:
        found = sorted(e["name"] for e in envs)
        print(f"fixture {name!r}: expected one match, found {found}", file=sys.stderr)
        return 1
    env = matches[0]
    if env["source_type"] != source_type:
        print(f"fixture {name!r}: source_type {env['source_type']!r}, expected {source_type!r}", file=sys.stderr)
        return 1
    # EnvironmentStatus serializes as a tagged object: {"status": "ready"}.
    status = env["status"]["status"]
    if status != "ready":
        print(f"fixture {name!r}: status {env['status']!r}, expected 'ready'", file=sys.stderr)
        return 1
    if env["python_version"].split(".")[:2] != expected.split("."):
        print(f"fixture {name!r}: listed Python {env['python_version']!r}, expected {expected}", file=sys.stderr)
        return 1
    python = Path(env["path"]) / "bin" / "python"
    version = subprocess.run(
        [str(python), "-c", "import sys; print('%d.%d' % sys.version_info[:2])"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if version != expected:
        print(f"fixture {name!r}: interpreter is {version}, expected {expected}", file=sys.stderr)
        return 1
    print(f"fixture {name!r} ({source_type}, Python {env['python_version']}): ready")

    # The dry run reads the source the way a real migration does (uv's
    # listing plus the env's own Python for package origins) and plans every
    # package, so a real conda / pyenv / virtualenvwrapper env that this
    # cannot read fails here instead of in a user's migration.
    dry = subprocess.run(
        [scuv, "migrate", "@env", name, "--dry-run", "--json"],
        capture_output=True,
        text=True,
    )
    if dry.returncode != 0:
        print(f"fixture {name!r}: migrate dry run failed:\n{dry.stdout}{dry.stderr}", file=sys.stderr)
        return 1
    plan = json.loads(dry.stdout)["data"]
    if plan["packages_failed"]:
        print(f"fixture {name!r}: dry run reports failures: {plan['packages_failed']}", file=sys.stderr)
        return 1
    excluded = [p["name"] for p in plan["packages_excluded"]]
    print(
        f"fixture {name!r}: dry run plans {plan['packages_migrated']} package(s), "
        f"{len(plan['packages_from_conda'])} from conda via the registry, excluded {excluded}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
