#!/usr/bin/env bash
# Migrating from pyenv-virtualenv to scoop
#
# This example demonstrates:
# - Detecting pyenv-virtualenv environments
# - Migrating selected environments
# - Migrating all environments at once
# - Handling naming conflicts

set -e

echo "=== pyenv-virtualenv to scoop Migration ==="
echo

# 1. Check if pyenv-virtualenv is available
echo "1. Checking for pyenv-virtualenv..."
if [ -d "$HOME/.pyenv/versions" ]; then
    echo "✅ pyenv-virtualenv detected"
    echo "   Location: $HOME/.pyenv/versions"
else
    echo "⚠️  pyenv-virtualenv not found"
    echo "   This example requires pyenv-virtualenv to demonstrate migration"
    exit 0
fi
echo

# 2. List available migrations
echo "2. Environments available for migration:"
scoop migrate list
echo

# 3. Migrate a single environment
echo "3. Migrating a single environment..."
echo "   Example: scoop migrate @myproject"
echo
echo "   This will:"
echo "   - Detect Python version from pyenv"
echo "   - Create equivalent environment in scoop"
echo "   - Preserve installed packages"
echo "   - Keep original pyenv env (safe operation)"
echo

# Uncomment to actually migrate (example environment name)
# scoop migrate @myproject

# 4. Migrate all environments
echo "4. Batch migration (all environments):"
echo "   Command: scoop migrate --all"
echo
echo "   This will:"
echo "   - Migrate all pyenv-virtualenv environments"
echo "   - Skip environments that already exist in scoop"
echo "   - Report success/failure for each"
echo

# Uncomment to migrate all
# scoop migrate --all

# 5. Handle naming conflicts
echo "5. Handling naming conflicts:"
echo "   If an environment already exists in scoop:"
echo "   - Migration will fail for that environment"
echo "   - Other environments will still be migrated"
echo "   - You can manually rename and retry"
echo
echo "   Example:"
echo "     scoop create myproject-new 3.12"
echo "     # Then migrate with a different source name if supported"
echo

# 6. Verify migrated environments
echo "6. Verify migration:"
scoop list
echo

# 7. Compare with pyenv
echo "7. Original pyenv environments:"
if command -v pyenv &> /dev/null; then
    pyenv versions --bare | grep -v "/"  # Filter out system Python
else
    echo "   (pyenv command not available)"
fi
echo

# 8. Test a migrated environment
echo "8. Testing migrated environment..."
echo "   scoop use myproject"
echo "   python --version"
echo "   pip list"
echo

# 9. Clean up old pyenv environments (manual step)
echo "9. Cleanup old pyenv environments (optional):"
echo "   ⚠️  Only after verifying scoop environments work!"
echo
echo "   pyenv uninstall <env-name>"
echo "   or"
echo "   rm -rf ~/.pyenv/versions/<env-name>"
echo

# 10. Update shell configuration
echo "10. Update shell configuration:"
echo "    Remove or comment out pyenv initialization:"
echo "    # eval \"\$(pyenv init -)\"              # ← Comment this out"
echo "    # eval \"\$(pyenv virtualenv-init -)\"   # ← Comment this out"
echo
echo "    Keep scoop initialization:"
echo "    eval \"\$(scoop init bash)\"            # ← Keep this"
echo

echo "=== Migration Guide Complete ==="
echo
echo "Pro tips:"
echo "  - Start with a single environment to test"
echo "  - Verify packages are preserved"
echo "  - Keep pyenv environments until confident"
echo "  - scoop is 100x+ faster than pyenv 🚀"
