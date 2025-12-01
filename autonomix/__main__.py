#!/usr/bin/env python3
"""Autonomix - A Linux package manager for GitHub releases."""

import sys
import os
from PySide6.QtWidgets import QApplication
from PySide6.QtGui import QIcon

from autonomix.ui.main_window import MainWindow
from autonomix.resources import get_icon_path


def main():
    """Main entry point for Autonomix."""
    app = QApplication(sys.argv)
    app.setApplicationName("Autonomix")
    app.setApplicationDisplayName("Autonomix")
    app.setOrganizationName("Autonomix")
    app.setDesktopFileName("autonomix")
    
    # Set application icon
    icon_path = get_icon_path()
    if os.path.exists(icon_path):
        app.setWindowIcon(QIcon(icon_path))
    
    # Set application style
    app.setStyle("Fusion")
    
    window = MainWindow()
    window.show()
    
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
