"""Assert that `scuv migrate list --json` found an image's fixture env.

Usage: scuv migrate list --json | python3 check_fixture.py <name> <source_type> <python>

<python> is the major.minor the fixture was created with, e.g. 3.12.

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
    name, source_type, expected = sys.argv[1], sys.argv[2], sys.argv[3]
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
    return 0


if __name__ == "__main__":
    sys.exit(main())
