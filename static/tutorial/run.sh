#!/bin/bash

# Navigate to the script's directory
cd "$(dirname "$0")"

# Define venv directory
VENV_DIR="venv"

# Check if venv exists
if [ ! -d "$VENV_DIR" ]; then
    echo "Creating virtual environment..."
    python3 -m venv $VENV_DIR
else
    echo "Virtual environment already exists."
fi

# Activate venv
source $VENV_DIR/bin/activate

# Install requirements
if [ -f "requirements.txt" ]; then
    echo "Installing requirements..."
    pip install -r requirements.txt
else
    echo "requirements.txt not found!"
    exit 1
fi

# Start Jupyter Lab
echo "Starting Jupyter Lab..."
jupyter lab
