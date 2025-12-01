"""Main window for Autonomix application."""

import tempfile
from pathlib import Path

from PySide6.QtWidgets import (
    QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel,
    QPushButton, QScrollArea, QFrame, QMessageBox, QProgressDialog,
    QToolBar, QStatusBar, QLineEdit, QApplication
)
from PySide6.QtCore import Qt, Signal, QThread, QObject, Slot
from PySide6.QtGui import QAction, QPalette, QColor

from ..core.github_api import GitHubAPI, ReleaseAsset
from ..core.database import Database, App
from ..core.installer import PackageInstaller
from .app_card import AppCard
from .add_app_dialog import AddAppDialog
from .styles import DARK_THEME, get_button_style, COLOR_SUCCESS, COLOR_WARNING, COLOR_PRIMARY


class InstallWorker(QObject):
    """Worker for installing packages in background."""
    
    progress = Signal(int, int)  # current, total
    finished = Signal(bool, str)  # success, message
    
    def __init__(self, github: GitHubAPI, installer: PackageInstaller,
                 asset: ReleaseAsset, package_type: str):
        super().__init__()
        self.github = github
        self.installer = installer
        self.asset = asset
        self.package_type = package_type
        self.install_path = None
    
    def run(self):
        try:
            # Download to temp directory
            with tempfile.TemporaryDirectory() as tmpdir:
                def progress_callback(current, total):
                    self.progress.emit(current, total)
                
                filepath = self.github.download_asset(
                    self.asset, tmpdir, progress_callback
                )
                
                # Install the package
                self.install_path = self.installer.install(
                    filepath, self.package_type
                )
                
                self.finished.emit(True, self.install_path)
                
        except Exception as e:
            self.finished.emit(False, str(e))


class UpdateChecker(QObject):
    """Worker for checking updates in background."""
    
    update_found = Signal(int, str)  # app_id, new_version
    finished = Signal()
    
    def __init__(self, github: GitHubAPI, apps: list[App]):
        super().__init__()
        self.github = github
        self.apps = apps
    
    def run(self):
        for app in self.apps:
            try:
                release = self.github.get_latest_release(app.owner, app.repo)
                if release and release.version != app.latest_version:
                    self.update_found.emit(app.id, release.version)
            except Exception:
                pass  # Skip failed checks
        
        self.finished.emit()


class MainWindow(QMainWindow):
    """Main application window."""
    
    def __init__(self):
        super().__init__()
        
        self.github = GitHubAPI()
        self.db = Database()
        self.installer = PackageInstaller()
        self.app_cards: dict[int, AppCard] = {}
        
        self.setWindowTitle("Autonomix")
        self.setMinimumSize(800, 600)
        self._setup_ui()
        self._load_apps()
        self._check_for_updates()
    
    def _setup_ui(self):
        # Apply dark theme globally
        self.setStyleSheet(DARK_THEME)
        
        # Set dark palette for native dialogs
        palette = QPalette()
        palette.setColor(QPalette.Window, QColor("#1e1e1e"))
        palette.setColor(QPalette.WindowText, QColor("#e0e0e0"))
        palette.setColor(QPalette.Base, QColor("#2d2d2d"))
        palette.setColor(QPalette.AlternateBase, QColor("#3c3c3c"))
        palette.setColor(QPalette.ToolTipBase, QColor("#2d2d2d"))
        palette.setColor(QPalette.ToolTipText, QColor("#e0e0e0"))
        palette.setColor(QPalette.Text, QColor("#e0e0e0"))
        palette.setColor(QPalette.Button, QColor("#3c3c3c"))
        palette.setColor(QPalette.ButtonText, QColor("#e0e0e0"))
        palette.setColor(QPalette.BrightText, QColor("#ffffff"))
        palette.setColor(QPalette.Link, QColor("#007acc"))
        palette.setColor(QPalette.Highlight, QColor("#007acc"))
        palette.setColor(QPalette.HighlightedText, QColor("#ffffff"))
        QApplication.instance().setPalette(palette)
        
        # Toolbar
        toolbar = QToolBar()
        toolbar.setMovable(False)
        self.addToolBar(toolbar)
        
        add_action = QAction("➕ Add App", self)
        add_action.triggered.connect(self._show_add_dialog)
        toolbar.addAction(add_action)
        
        refresh_action = QAction("🔄 Check Updates", self)
        refresh_action.triggered.connect(self._check_for_updates)
        toolbar.addAction(refresh_action)
        
        toolbar.addSeparator()
        
        # Search box
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Search apps...")
        self.search_input.setMaximumWidth(300)
        self.search_input.textChanged.connect(self._filter_apps)
        toolbar.addWidget(self.search_input)
        
        # Central widget
        central = QWidget()
        self.setCentralWidget(central)
        
        layout = QVBoxLayout(central)
        layout.setContentsMargins(16, 16, 16, 16)
        
        # Header
        header = QHBoxLayout()
        
        title = QLabel("Your Apps")
        title.setStyleSheet("font-size: 24px; font-weight: bold;")
        header.addWidget(title)
        
        header.addStretch()
        
        self.update_badge = QLabel()
        self.update_badge.setStyleSheet("""
            background-color: #ff9800;
            color: white;
            padding: 4px 12px;
            border-radius: 12px;
            font-weight: bold;
        """)
        self.update_badge.hide()
        header.addWidget(self.update_badge)
        
        layout.addLayout(header)
        
        # Scrollable app list
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)
        
        self.apps_container = QWidget()
        self.apps_layout = QVBoxLayout(self.apps_container)
        self.apps_layout.setSpacing(8)
        self.apps_layout.setAlignment(Qt.AlignTop)
        
        scroll.setWidget(self.apps_container)
        layout.addWidget(scroll)
        
        # Empty state
        self.empty_state = QFrame()
        empty_layout = QVBoxLayout(self.empty_state)
        empty_layout.setAlignment(Qt.AlignCenter)
        
        empty_label = QLabel("No apps added yet")
        empty_label.setStyleSheet("font-size: 18px; color: #888;")
        empty_label.setAlignment(Qt.AlignCenter)
        empty_layout.addWidget(empty_label)
        
        empty_hint = QLabel("Click 'Add App' to start tracking GitHub releases")
        empty_hint.setStyleSheet("color: #666;")
        empty_hint.setAlignment(Qt.AlignCenter)
        empty_layout.addWidget(empty_hint)
        
        self.apps_layout.addWidget(self.empty_state)
        
        # Status bar
        self.statusBar().showMessage("Ready")
    
    def _load_apps(self):
        """Load all apps from database and display them."""
        apps = self.db.get_all_apps()
        
        self.empty_state.setVisible(len(apps) == 0)
        
        for app in apps:
            self._add_app_card(app)
    
    def _add_app_card(self, app: App):
        """Add an app card to the UI."""
        card = AppCard(app)
        
        # If the card detected different installation status, sync to database
        if card.app.installed_version != app.installed_version:
            app.installed_version = card.app.installed_version
            self.db.update_app(app)
        
        card.install_requested.connect(self._install_app)
        card.update_requested.connect(self._update_app)
        card.uninstall_requested.connect(self._uninstall_app)
        card.remove_requested.connect(self._remove_app)
        card.details_requested.connect(self._show_app_details)
        
        self.apps_layout.insertWidget(self.apps_layout.count() - 1, card)
        self.app_cards[app.id] = card
        
        self.empty_state.hide()
    
    def _show_add_dialog(self):
        """Show the add app dialog."""
        dialog = AddAppDialog(self)
        dialog.app_added.connect(self._on_app_added)
        dialog.exec()
    
    @Slot(dict)
    def _on_app_added(self, app_data: dict):
        """Handle a new app being added."""
        from datetime import datetime
        
        # Check if app is already installed on the system
        is_installed, installed_version = self.installer.check_app_status(
            app_data['name'], app_data['package_type']
        )
        
        app = App(
            id=None,
            name=app_data['name'],
            repo_url=app_data['repo_url'],
            owner=app_data['owner'],
            repo=app_data['repo'],
            installed_version=installed_version if is_installed else None,
            latest_version=app_data['latest_version'],
            package_type=app_data['package_type'],
            install_path=None,
            added_at=datetime.now().isoformat(),
            updated_at=datetime.now().isoformat(),
            auto_update=app_data.get('auto_update', False),
            include_prerelease=app_data.get('include_prerelease', False),
        )
        
        app_id = self.db.add_app(app)
        app.id = app_id
        
        self._add_app_card(app)
        
        if is_installed:
            self.statusBar().showMessage(f"Added {app.name} (already installed: {installed_version})")
        else:
            self.statusBar().showMessage(f"Added {app.name}")
        
        # Store asset for potential installation
        if 'asset' in app_data and app_data['asset']:
            self._pending_assets = getattr(self, '_pending_assets', {})
            self._pending_assets[app_id] = app_data['asset']
    
    def _install_app(self, app: App):
        """Install an app."""
        # Get the asset for this app
        asset = getattr(self, '_pending_assets', {}).get(app.id)
        
        if not asset:
            # Fetch the latest release
            try:
                release = self.github.get_latest_release(app.owner, app.repo)
                if not release:
                    QMessageBox.warning(self, "Error", "No releases found")
                    return
                
                asset = self.installer.select_best_asset(
                    release.assets, app.package_type
                )
                if not asset:
                    QMessageBox.warning(self, "Error", 
                                       f"No {app.package_type} package found")
                    return
            except Exception as e:
                QMessageBox.warning(self, "Error", str(e))
                return
        
        # Show progress dialog
        progress = QProgressDialog(
            f"Installing {app.name}...", "Cancel", 0, 100, self
        )
        progress.setWindowModality(Qt.WindowModal)
        progress.setAutoClose(True)
        progress.show()
        
        # Run installation in thread
        self.install_thread = QThread()
        self.install_worker = InstallWorker(
            self.github, self.installer, asset, app.package_type
        )
        self.install_worker.moveToThread(self.install_thread)
        
        self.install_thread.started.connect(self.install_worker.run)
        self.install_worker.progress.connect(
            lambda c, t: progress.setValue(int(c / t * 100))
        )
        
        def on_finished(success, message):
            self.install_thread.quit()
            progress.close()
            
            if success:
                app.installed_version = app.latest_version
                app.install_path = message
                self.db.update_app(app)
                
                if app.id in self.app_cards:
                    self.app_cards[app.id].update_app(app)
                
                self.statusBar().showMessage(f"Installed {app.name}")
                self._update_badge()
            else:
                QMessageBox.warning(self, "Installation Failed", message)
        
        self.install_worker.finished.connect(on_finished)
        self.install_worker.finished.connect(self.install_thread.quit)
        
        self.install_thread.start()
    
    def _update_app(self, app: App):
        """Update an app to the latest version."""
        reply = QMessageBox.question(
            self, "Update App",
            f"Update {app.name} from {app.installed_version} to {app.latest_version}?",
            QMessageBox.Yes | QMessageBox.No
        )
        
        if reply == QMessageBox.Yes:
            self._install_app(app)
    
    def _uninstall_app(self, app: App):
        """Uninstall an app."""
        reply = QMessageBox.question(
            self, "Uninstall App",
            f"Are you sure you want to uninstall {app.name}?",
            QMessageBox.Yes | QMessageBox.No
        )
        
        if reply == QMessageBox.Yes:
            success = self.installer.uninstall(
                app.package_type, app.name, app.install_path
            )
            
            if success:
                app.installed_version = None
                app.install_path = None
                self.db.update_app(app)
                
                if app.id in self.app_cards:
                    self.app_cards[app.id].update_app(app)
                
                self.statusBar().showMessage(f"Uninstalled {app.name}")
            else:
                QMessageBox.warning(
                    self, "Uninstall Failed",
                    f"Could not uninstall {app.name}. You may need to remove it manually."
                )
    
    def _remove_app(self, app: App):
        """Remove an app from the tracking list."""
        reply = QMessageBox.question(
            self, "Remove App",
            f"Remove {app.name} from tracking? This will not uninstall the app.",
            QMessageBox.Yes | QMessageBox.No
        )
        
        if reply == QMessageBox.Yes:
            self.db.delete_app(app.id)
            
            if app.id in self.app_cards:
                card = self.app_cards.pop(app.id)
                card.deleteLater()
            
            if not self.app_cards:
                self.empty_state.show()
            
            self.statusBar().showMessage(f"Removed {app.name}")
    
    def _show_app_details(self, app: App):
        """Show detailed information about an app."""
        details = f"""
<h2>{app.name}</h2>
<p><b>Repository:</b> {app.owner}/{app.repo}</p>
<p><b>Installed Version:</b> {app.installed_version or 'Not installed'}</p>
<p><b>Latest Version:</b> {app.latest_version or 'Unknown'}</p>
<p><b>Package Type:</b> {app.package_type}</p>
<p><b>Install Path:</b> {app.install_path or 'N/A'}</p>
<p><b>Added:</b> {app.added_at}</p>
<p><b>Last Updated:</b> {app.updated_at}</p>
"""
        msg = QMessageBox(self)
        msg.setWindowTitle("App Details")
        msg.setTextFormat(Qt.RichText)
        msg.setText(details)
        msg.exec()
    
    def _check_for_updates(self):
        """Check all apps for updates."""
        apps = self.db.get_all_apps()
        if not apps:
            return
        
        self.statusBar().showMessage("Checking for updates...")
        
        self.update_thread = QThread()
        self.update_worker = UpdateChecker(self.github, apps)
        self.update_worker.moveToThread(self.update_thread)
        
        self.update_thread.started.connect(self.update_worker.run)
        self.update_worker.update_found.connect(self._on_update_found)
        self.update_worker.finished.connect(self._on_update_check_complete)
        self.update_worker.finished.connect(self.update_thread.quit)
        
        self.update_thread.start()
    
    @Slot(int, str)
    def _on_update_found(self, app_id: int, new_version: str):
        """Handle an update being found."""
        app = self.db.get_app(app_id)
        if app:
            app.latest_version = new_version
            self.db.update_app(app)
            
            if app_id in self.app_cards:
                self.app_cards[app_id].update_app(app)
    
    @Slot()
    def _on_update_check_complete(self):
        """Handle update check completion."""
        self._update_badge()
        self.statusBar().showMessage("Update check complete")
    
    def _update_badge(self):
        """Update the updates available badge."""
        apps = self.db.get_all_apps()
        updates = sum(1 for app in apps if app.has_update)
        
        if updates > 0:
            self.update_badge.setText(f"{updates} update{'s' if updates > 1 else ''} available")
            self.update_badge.show()
        else:
            self.update_badge.hide()
    
    def _filter_apps(self, text: str):
        """Filter displayed apps by search text."""
        text = text.lower()
        
        for app_id, card in self.app_cards.items():
            visible = (
                text in card.app.name.lower() or
                text in card.app.owner.lower() or
                text in card.app.repo.lower()
            )
            card.setVisible(visible)
