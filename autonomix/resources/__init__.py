"""Resources for Autonomix."""

import os

RESOURCES_DIR = os.path.dirname(__file__)

def get_icon_path():
    """Get the path to the application icon."""
    return os.path.join(RESOURCES_DIR, 'autonomix.svg')
