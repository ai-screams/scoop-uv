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

## Can I use scoop with conda environments?

Not directly. They serve different purposes and operate independently:

**conda** is a *package and environment manager*. It handles:
- Its own binaries and non-Python dependencies
- Heavy data science libraries (MKL, CUDA, cuDNN, etc.)

**scoop** is a *lightweight environment manager* powered by [uv](https://github.com/astral-sh/uv). It:
- Leverages your existing Python installations
- Creates fast, portable virtual environments

> **When to use what:** For heavy data science requiring non-Python libraries → conda. For almost everything else → scoop (significantly faster and more portable).
