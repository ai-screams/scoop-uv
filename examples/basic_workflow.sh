#!/usr/bin/env bash
# Basic scoop workflow - Creating and using virtual environments
#
# This example demonstrates:
# - Installing Python versions
# - Creating virtual environments
# - Using environments in a project
# - Installing packages
# - Cleaning up

set -e  # Exit on error

echo "=== scoop Basic Workflow Example ==="
echo

# 1. Check scoop installation
echo "1. Checking scoop installation..."
scoop --version
echo "✅ scoop is installed"
echo

# 2. Install Python 3.12 if not already installed
echo "2. Installing Python 3.12..."
if scoop list --pythons --json | grep -q "3.12"; then
    echo "ℹ️  Python 3.12 already installed"
else
    scoop install 3.12
    echo "✅ Python 3.12 installed"
fi
echo

# 3. Create a new virtual environment
echo "3. Creating virtual environment 'demo-project'..."
if scoop list --json | grep -q "demo-project"; then
    echo "ℹ️  Environment 'demo-project' already exists, skipping"
else
    scoop create demo-project 3.12
    echo "✅ Environment created"
fi
echo

# 4. View environment details
echo "4. Environment information:"
scoop info demo-project
echo

# 5. Set environment for current directory
echo "5. Setting 'demo-project' as active environment..."
scoop use demo-project
echo "✅ Environment set (will auto-activate on cd)"
echo

# 6. Verify activation (in a subshell to simulate cd behavior)
echo "6. Verifying activation..."
(
    # Re-source scoop to trigger auto-activation
    eval "$(scoop init bash)"

    if [ -n "$VIRTUAL_ENV" ]; then
        echo "✅ Environment is active: $SCOOP_ACTIVE"
        echo "   Python: $(python --version)"
        echo "   pip: $(pip --version)"
    else
        echo "⚠️  Environment not activated (you may need to cd into the directory)"
    fi
)
echo

# 7. Install a package (demonstrates package isolation)
echo "7. Installing requests package..."
(
    # Activate environment
    eval "$(scoop shell demo-project)"

    pip install requests --quiet
    echo "✅ requests installed"

    # Verify installation
    python -c "import requests; print(f'   requests version: {requests.__version__}')"
)
echo

# 8. List all environments
echo "8. All environments:"
scoop list
echo

# 9. Check environment health
echo "9. Running health check..."
scoop doctor
echo

# 10. Clean up (optional - uncomment to delete)
echo "10. Cleanup (environment will be kept for demonstration)"
echo "    To remove: scoop remove demo-project"
# scoop remove demo-project
echo

echo "=== Example Complete ==="
echo
echo "Next steps:"
echo "  - cd into your project directory"
echo "  - Run: scoop use demo-project"
echo "  - Environment will auto-activate on directory entry"
echo "  - Start coding! 🍨"
