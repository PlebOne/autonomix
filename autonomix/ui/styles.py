"""Shared styles for Autonomix UI."""

DARK_THEME = """
/* Global */
QWidget {
    background-color: #1e1e1e;
    color: #e0e0e0;
    font-family: 'Segoe UI', 'Ubuntu', 'Noto Sans', sans-serif;
    font-size: 13px;
}

/* Main Window */
QMainWindow {
    background-color: #1e1e1e;
}

/* Toolbar */
QToolBar {
    background-color: #252526;
    border: none;
    padding: 8px;
    spacing: 8px;
}

QToolBar QToolButton {
    background-color: #3c3c3c;
    color: #e0e0e0;
    border: none;
    padding: 6px 12px;
    border-radius: 4px;
    margin: 2px;
}

QToolBar QToolButton:hover {
    background-color: #4a4a4a;
}

QToolBar QToolButton:pressed {
    background-color: #555555;
}

/* Status Bar */
QStatusBar {
    background-color: #007acc;
    color: white;
    padding: 4px;
}

/* Scroll Area */
QScrollArea {
    border: none;
    background-color: transparent;
}

QScrollBar:vertical {
    background-color: #2d2d2d;
    width: 12px;
    border-radius: 6px;
}

QScrollBar::handle:vertical {
    background-color: #555555;
    min-height: 30px;
    border-radius: 6px;
}

QScrollBar::handle:vertical:hover {
    background-color: #666666;
}

QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {
    height: 0px;
}

QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {
    background: none;
}

/* Line Edit */
QLineEdit {
    background-color: #3c3c3c;
    border: 1px solid #555555;
    border-radius: 4px;
    padding: 8px 12px;
    color: #e0e0e0;
    selection-background-color: #007acc;
}

QLineEdit:focus {
    border-color: #007acc;
}

QLineEdit:disabled {
    background-color: #2d2d2d;
    color: #888888;
}

/* Combo Box */
QComboBox {
    background-color: #3c3c3c;
    border: 1px solid #555555;
    border-radius: 4px;
    padding: 6px 12px;
    color: #e0e0e0;
    min-width: 150px;
}

QComboBox:hover {
    border-color: #666666;
}

QComboBox:focus {
    border-color: #007acc;
}

QComboBox::drop-down {
    border: none;
    width: 24px;
}

QComboBox::down-arrow {
    image: none;
    border-left: 5px solid transparent;
    border-right: 5px solid transparent;
    border-top: 6px solid #888888;
    margin-right: 8px;
}

QComboBox QAbstractItemView {
    background-color: #2d2d2d;
    border: 1px solid #555555;
    color: #e0e0e0;
    selection-background-color: #007acc;
    padding: 4px;
}

/* Check Box */
QCheckBox {
    color: #e0e0e0;
    spacing: 8px;
}

QCheckBox::indicator {
    width: 18px;
    height: 18px;
    border-radius: 3px;
    border: 1px solid #555555;
    background-color: #3c3c3c;
}

QCheckBox::indicator:checked {
    background-color: #007acc;
    border-color: #007acc;
}

QCheckBox::indicator:hover {
    border-color: #007acc;
}

/* Push Button */
QPushButton {
    background-color: #3c3c3c;
    color: #e0e0e0;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    font-weight: bold;
}

QPushButton:hover {
    background-color: #4a4a4a;
}

QPushButton:pressed {
    background-color: #555555;
}

QPushButton:disabled {
    background-color: #2d2d2d;
    color: #666666;
}

/* Labels */
QLabel {
    color: #e0e0e0;
    background-color: transparent;
}

/* Frame */
QFrame {
    background-color: transparent;
}

/* Progress Bar */
QProgressBar {
    background-color: #3c3c3c;
    border: none;
    border-radius: 4px;
    height: 8px;
    text-align: center;
}

QProgressBar::chunk {
    background-color: #007acc;
    border-radius: 4px;
}

/* Dialog */
QDialog {
    background-color: #1e1e1e;
}

/* Message Box */
QMessageBox {
    background-color: #1e1e1e;
}

QMessageBox QLabel {
    color: #e0e0e0;
}

QMessageBox QPushButton {
    min-width: 80px;
    padding: 6px 16px;
}

/* Progress Dialog */
QProgressDialog {
    background-color: #1e1e1e;
}

/* Menu */
QMenu {
    background-color: #2d2d2d;
    color: #e0e0e0;
    border: 1px solid #444444;
    padding: 4px;
}

QMenu::item {
    padding: 6px 24px;
    border-radius: 2px;
}

QMenu::item:selected {
    background-color: #007acc;
}

QMenu::separator {
    height: 1px;
    background-color: #444444;
    margin: 4px 8px;
}

/* Tooltip */
QToolTip {
    background-color: #2d2d2d;
    color: #e0e0e0;
    border: 1px solid #555555;
    padding: 4px 8px;
}
"""


def get_button_style(color: str) -> str:
    """Get a styled button with the given color."""
    return f"""
        QPushButton {{
            background-color: {color};
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            font-weight: bold;
        }}
        QPushButton:hover {{
            background-color: {color};
            opacity: 0.8;
        }}
        QPushButton:pressed {{
            background-color: {color};
            opacity: 0.6;
        }}
        QPushButton:disabled {{
            background-color: #555555;
            color: #888888;
        }}
    """


# Color constants
COLOR_PRIMARY = "#007acc"
COLOR_SUCCESS = "#4caf50"
COLOR_WARNING = "#ff9800"
COLOR_DANGER = "#f44336"
COLOR_NEUTRAL = "#555555"
