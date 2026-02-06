# scoop Examples

Real-world usage scenarios for scoop, the Python virtual environment manager.

## Quick Reference

| Example | Description |
|---------|-------------|
| [basic_workflow.sh](basic_workflow.sh) | Creating, using, and managing environments |
| [migration_from_pyenv.sh](migration_from_pyenv.sh) | Migrating from pyenv-virtualenv |
| [multi_project_setup.sh](multi_project_setup.sh) | Managing multiple projects with different Python versions |
| [ci_github_actions.yml](ci_github_actions.yml) | GitHub Actions CI integration |

## Prerequisites

All examples assume you have:
1. scoop installed (`cargo install scoop-uv`)
2. Shell integration enabled (`eval "$(scoop init bash)"`)
3. uv installed (`curl -LsSf https://astral.sh/uv/install.sh | sh`)

## Running Examples

```bash
# Make examples executable
chmod +x examples/*.sh

# Run an example
./examples/basic_workflow.sh
```

## Contributing Examples

Have a useful scoop workflow? Contribute an example!

1. Create a new file in `examples/`
2. Add clear comments explaining each step
3. Include expected output where helpful
4. Update this README
5. Submit a PR

See [CONTRIBUTING.md](../docs/src/development/contributing.md) for details.
