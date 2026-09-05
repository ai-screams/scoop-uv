#!/usr/bin/env bash
# Managing multiple projects with different Python versions
#
# Scenario: Developer working on 3 projects simultaneously
# - legacy-api: Python 3.10
# - current-webapp: Python 3.11
# - next-gen: Python 3.12

set -e

echo "=== Multi-Project Setup with scuv ==="
echo

# 1. Setup project directories
echo "1. Creating project structure..."
mkdir -p ~/Projects/{legacy-api,current-webapp,next-gen}
echo "   ~/Projects/"
echo "   ├── legacy-api/      (Python 3.10)"
echo "   ├── current-webapp/  (Python 3.11)"
echo "   └── next-gen/        (Python 3.12)"
echo

# 2. Install required Python versions
echo "2. Installing Python versions..."
for version in 3.10 3.11 3.12; do
    if scuv list --pythons --json | grep -q "$version"; then
        echo "   ✅ Python $version already installed"
    else
        scuv install $version
        echo "   ✅ Python $version installed"
    fi
done
echo

# 3. Create environments for each project
echo "3. Creating virtual environments..."

# Legacy API - Python 3.10
cd ~/Projects/legacy-api
if [ ! -f .scuv-version ]; then
    scuv create legacy-api 3.10
    scuv use legacy-api
    echo "   ✅ legacy-api → Python 3.10"

    # Install dependencies
    cat > requirements.txt << 'EOF'
flask==2.0.3
sqlalchemy==1.4.46
requests==2.28.2
EOF
    eval "$(scuv shell legacy-api)"
    pip install -r requirements.txt --quiet
fi

# Current WebApp - Python 3.11
cd ~/Projects/current-webapp
if [ ! -f .scuv-version ]; then
    scuv create current-webapp 3.11
    scuv use current-webapp
    echo "   ✅ current-webapp → Python 3.11"

    # Install dependencies
    cat > requirements.txt << 'EOF'
fastapi==0.104.1
uvicorn==0.24.0
pydantic==2.5.0
EOF
    eval "$(scuv shell current-webapp)"
    pip install -r requirements.txt --quiet
fi

# Next-Gen - Python 3.12
cd ~/Projects/next-gen
if [ ! -f .scuv-version ]; then
    scuv create next-gen 3.12
    scuv use next-gen
    echo "   ✅ next-gen → Python 3.12"

    # Install dependencies
    cat > requirements.txt << 'EOF'
django==5.0
psycopg==3.1.16
celery==5.3.4
EOF
    eval "$(scuv shell next-gen)"
    pip install -r requirements.txt --quiet
fi
echo

# 4. Demonstrate auto-activation
echo "4. Testing auto-activation..."
echo

echo "   Switching to legacy-api:"
cd ~/Projects/legacy-api
eval "$(scuv init bash)"  # Re-init to trigger auto-activation
echo "   Active: $(cat .scuv-version)"
(
    eval "$(scuv shell legacy-api)"
    echo "   Python: $(python --version)"
    echo "   Flask: $(python -c 'import flask; print(flask.__version__)')"
)
echo

echo "   Switching to current-webapp:"
cd ~/Projects/current-webapp
echo "   Active: $(cat .scuv-version)"
(
    eval "$(scuv shell current-webapp)"
    echo "   Python: $(python --version)"
    echo "   FastAPI: $(python -c 'import fastapi; print(fastapi.__version__)')"
)
echo

echo "   Switching to next-gen:"
cd ~/Projects/next-gen
echo "   Active: $(cat .scuv-version)"
(
    eval "$(scuv shell next-gen)"
    echo "   Python: $(python --version)"
    echo "   Django: $(python -c 'import django; print(django.__version__)')"
)
echo

# 5. List all project environments
echo "5. All project environments:"
scuv list
echo

# 6. IDE integration (symlink for each project)
echo "6. IDE integration (optional)..."
echo "   Creating .venv symlinks for IDE recognition:"

cd ~/Projects/legacy-api
scuv use legacy-api --link
echo "   ✅ legacy-api/.venv → ~/.scuv/virtualenvs/legacy-api"

cd ~/Projects/current-webapp
scuv use current-webapp --link
echo "   ✅ current-webapp/.venv → ~/.scuv/virtualenvs/current-webapp"

cd ~/Projects/next-gen
scuv use next-gen --link
echo "   ✅ next-gen/.venv → ~/.scuv/virtualenvs/next-gen"
echo

# 7. Workspace switcher function (add to ~/.bashrc)
echo "7. Workspace switcher (add to ~/.bashrc)..."
cat << 'EOF'

# Quick project switcher
alias work-legacy="cd ~/Projects/legacy-api"
alias work-webapp="cd ~/Projects/current-webapp"
alias work-nextgen="cd ~/Projects/next-gen"

# Or generic function
work() {
    case "$1" in
        legacy)   cd ~/Projects/legacy-api ;;
        webapp)   cd ~/Projects/current-webapp ;;
        nextgen)  cd ~/Projects/next-gen ;;
        *)        echo "Usage: work {legacy|webapp|nextgen}" ;;
    esac
}
EOF
echo

# 8. Health check all environments
echo "8. Running health check..."
scuv doctor
echo

# 9. Usage summary
echo "=== Setup Complete ==="
echo
echo "Workflow:"
echo "  1. cd ~/Projects/legacy-api    # Auto-activates legacy-api env"
echo "  2. python run.py               # Uses Python 3.10 + Flask"
echo "  3. cd ~/Projects/current-webapp # Auto-activates current-webapp env"
echo "  4. uvicorn main:app            # Uses Python 3.11 + FastAPI"
echo "  5. cd ~/Projects/next-gen      # Auto-activates next-gen env"
echo "  6. python manage.py runserver  # Uses Python 3.12 + Django"
echo
echo "Pro tips:"
echo "  - No manual activation needed!"
echo "  - .venv symlinks work with VS Code, PyCharm"
echo "  - Each project isolated, zero conflicts"
echo "  - Switch projects instantly 🚀"
