# Frequently Asked Questions

## What's the difference between scoop and pyenv?

While both tools help you manage Python, they focus on different parts of the workflow:

**pyenv** is primarily a *version manager*. It focuses on:
- Installing multiple versions of the Python interpreter (e.g., 3.9.0, 3.12.1)
- Switching between them globally or per folder

**scoop** is an *environment and workflow manager* powered by [uv](https://github.com/astral-sh/uv). It focuses on:
- Creating and managing isolated virtual environments
- Fast project-specific environment workflows

> **Summary:** You might use pyenv to install Python 3.11 on your machine, but you use scoop to actually build and run your application within a lightning-fast virtual environment using that Python version.
