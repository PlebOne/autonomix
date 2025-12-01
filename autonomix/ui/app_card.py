"""App card widget for displaying application information."""

from PySide6.QtWidgets import (
    QFrame, QVBoxLayout, QHBoxLayout, QLabel, QPushButton,
    QSizePolicy, QMenu, QWidget
)
from PySide6.QtCore import Signal, Qt
from PySide6.QtGui import QAction

from ..core.database import App
from ..core.installer import PackageInstaller
from .styles import get_button_style, COLOR_SUCCESS, COLOR_WARNING, COLOR_NEUTRAL, COLOR_PRIMARY, COLOR_DANGER


class AppCard(QFrame):
    """Card widget displaying an application's info and actions."""
    
    install_requested = Signal(App)
    update_requested = Signal(App)
    uninstall_requested = Signal(App)
    remove_requested = Signal(App)
    details_requested = Signal(App)
    
    # Shared installer instance for checking installation status
    _installer = None
    
    @classmethod
    def get_installer(cls):
        if cls._installer is None:
            cls._installer = PackageInstaller()
        return cls._installer
    
    def __init__(self, app: App, parent=None):
        super().__init__(parent)
        self.app = app
        self._check_installation_status()
        self._setup_ui()
    
    def _check_installation_status(self):
        """Check if the app is actually installed on the system."""
        installer = self.get_installer()
        is_installed, version = installer.check_app_status(
            self.app.name, self.app.package_type
        )
        
        if is_installed and not self.app.installed_version:
            # App is installed but we didn't know - update our record
            self.app.installed_version = version or "installed"
        elif not is_installed and self.app.installed_version:
            # App was uninstalled externally
            self.app.installed_version = None
    
    def _setup_ui(self):
        self.setObjectName("appCard")
        self.setStyleSheet("""
            QFrame#appCard {
                background-color: #2d2d2d;
                border-radius: 8px;
                border: 1px solid #3c3c3c;
            }
            QFrame#appCard:hover {
                background-color: #333333;
                border-color: #4a4a4a;
            }
        """)
        
        self.main_layout = QVBoxLayout(self)
        self.main_layout.setContentsMargins(16, 12, 16, 12)
        self.main_layout.setSpacing(8)
        
        self._build_content()
    
    def _build_content(self):
        """Build or rebuild all content."""
        # Clear existing content
        while self.main_layout.count():
            item = self.main_layout.takeAt(0)
            if item.widget():
                item.widget().deleteLater()
            elif item.layout():
                self._clear_layout(item.layout())
        
        # Header row (name + status)
        header = QHBoxLayout()
        
        self.name_label = QLabel(self.app.name)
        self.name_label.setStyleSheet("font-size: 16px; font-weight: bold; color: #ffffff; background: transparent;")
        header.addWidget(self.name_label)
        
        header.addStretch()
        
        self.status_label = QLabel()
        self._update_status_label()
        header.addWidget(self.status_label)
        
        header_widget = QWidget()
        header_widget.setLayout(header)
        header_widget.setStyleSheet("background: transparent;")
        self.main_layout.addWidget(header_widget)
        
        # Repository info
        repo_label = QLabel(f"{self.app.owner}/{self.app.repo}")
        repo_label.setStyleSheet("color: #888888; font-size: 12px; background: transparent;")
        self.main_layout.addWidget(repo_label)
        
        # Version info row
        version_layout = QHBoxLayout()
        
        self.installed_label = QLabel()
        self.installed_label.setStyleSheet("color: #aaaaaa; background: transparent;")
        if self.app.installed_version:
            self.installed_label.setText(f"Installed: {self.app.installed_version}")
        else:
            self.installed_label.setText("Not installed")
        version_layout.addWidget(self.installed_label)
        
        if self.app.latest_version:
            self.latest_label = QLabel(f"Latest: {self.app.latest_version}")
            color = COLOR_SUCCESS if not self.app.has_update else COLOR_WARNING
            self.latest_label.setStyleSheet(f"color: {color}; background: transparent;")
            version_layout.addWidget(self.latest_label)
        
        version_layout.addStretch()
        
        # Package type badge
        type_label = QLabel(self.app.package_type.upper())
        type_label.setStyleSheet(f"""
            background-color: {COLOR_PRIMARY};
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 11px;
        """)
        version_layout.addWidget(type_label)
        
        version_widget = QWidget()
        version_widget.setLayout(version_layout)
        version_widget.setStyleSheet("background: transparent;")
        self.main_layout.addWidget(version_widget)
        
        # Action buttons
        button_layout = QHBoxLayout()
        button_layout.addStretch()
        
        if not self.app.installed_version:
            # Not installed - show Install button
            install_btn = QPushButton("Install")
            install_btn.setStyleSheet(get_button_style(COLOR_SUCCESS))
            install_btn.clicked.connect(lambda: self.install_requested.emit(self.app))
            button_layout.addWidget(install_btn)
        else:
            # Installed - show Uninstall button
            uninstall_btn = QPushButton("Uninstall")
            uninstall_btn.setStyleSheet(get_button_style(COLOR_DANGER))
            uninstall_btn.clicked.connect(lambda: self.uninstall_requested.emit(self.app))
            button_layout.addWidget(uninstall_btn)
            
            # Also show Update button if update available
            if self.app.has_update:
                update_btn = QPushButton("Update")
                update_btn.setStyleSheet(get_button_style(COLOR_WARNING))
                update_btn.clicked.connect(lambda: self.update_requested.emit(self.app))
                button_layout.addWidget(update_btn)
        
        # More options menu
        self.more_btn = QPushButton("⋮")
        self.more_btn.setFixedWidth(36)
        self.more_btn.setStyleSheet(get_button_style(COLOR_NEUTRAL))
        self.more_btn.clicked.connect(self._show_menu)
        button_layout.addWidget(self.more_btn)
        
        button_widget = QWidget()
        button_widget.setLayout(button_layout)
        button_widget.setStyleSheet("background: transparent;")
        self.main_layout.addWidget(button_widget)
    
    def _clear_layout(self, layout):
        """Recursively clear a layout."""
        while layout.count():
            item = layout.takeAt(0)
            if item.widget():
                item.widget().deleteLater()
            elif item.layout():
                self._clear_layout(item.layout())
    
    def _update_status_label(self):
        if not self.app.installed_version:
            self.status_label.setText("Not installed")
            self.status_label.setStyleSheet("color: #888888; background: transparent;")
        elif self.app.has_update:
            self.status_label.setText("⬆ Update available")
            self.status_label.setStyleSheet(f"color: {COLOR_WARNING}; background: transparent;")
        else:
            self.status_label.setText("✓ Installed")
            self.status_label.setStyleSheet(f"color: {COLOR_SUCCESS}; background: transparent;")
    
    def _show_menu(self):
        menu = QMenu(self)
        
        details_action = menu.addAction("View Details")
        details_action.triggered.connect(lambda: self.details_requested.emit(self.app))
        
        menu.addSeparator()
        
        if self.app.installed_version:
            reinstall_action = menu.addAction("Reinstall")
            reinstall_action.triggered.connect(lambda: self.install_requested.emit(self.app))
        
        menu.addSeparator()
        
        remove_action = menu.addAction("Remove from list")
        remove_action.triggered.connect(lambda: self.remove_requested.emit(self.app))
        
        menu.exec_(self.more_btn.mapToGlobal(self.more_btn.rect().bottomLeft()))
    
    def update_app(self, app: App):
        """Update the displayed app data - rebuilds the entire card."""
        self.app = app
        self._check_installation_status()
        self._build_content()
