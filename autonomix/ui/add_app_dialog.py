"""Dialog for adding a new application to track."""

from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QLabel, QLineEdit,
    QPushButton, QComboBox, QCheckBox, QFormLayout, QFrame,
    QMessageBox, QProgressBar
)
from PySide6.QtCore import Qt, Signal, QThread, QObject

from ..core.github_api import GitHubAPI, Release
from ..core.installer import PackageInstaller
from .styles import DARK_THEME, get_button_style, COLOR_PRIMARY, COLOR_SUCCESS, COLOR_NEUTRAL


class FetchWorker(QObject):
    """Worker for fetching release info in background."""
    
    finished = Signal(object, object)  # (release, error)
    
    def __init__(self, github: GitHubAPI, owner: str, repo: str):
        super().__init__()
        self.github = github
        self.owner = owner
        self.repo = repo
    
    def run(self):
        try:
            release = self.github.get_latest_release(self.owner, self.repo)
            self.finished.emit(release, None)
        except Exception as e:
            self.finished.emit(None, str(e))


class AddAppDialog(QDialog):
    """Dialog for adding a new GitHub repository to track."""
    
    app_added = Signal(dict)  # Emits app data when confirmed
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.github = GitHubAPI()
        self.installer = PackageInstaller()
        self.release = None
        self.selected_asset = None
        
        self.setWindowTitle("Add Application")
        self.setMinimumWidth(500)
        self.setStyleSheet(DARK_THEME)
        self._setup_ui()
    
    def _setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setSpacing(16)
        layout.setContentsMargins(20, 20, 20, 20)
        
        # URL input
        url_layout = QVBoxLayout()
        url_label = QLabel("GitHub Repository URL:")
        url_label.setStyleSheet("font-weight: bold; background: transparent;")
        url_layout.addWidget(url_label)
        
        input_layout = QHBoxLayout()
        self.url_input = QLineEdit()
        self.url_input.setPlaceholderText("https://github.com/owner/repo or owner/repo")
        input_layout.addWidget(self.url_input)
        
        self.fetch_btn = QPushButton("Fetch")
        self.fetch_btn.setStyleSheet(get_button_style(COLOR_PRIMARY))
        self.fetch_btn.clicked.connect(self._fetch_release)
        input_layout.addWidget(self.fetch_btn)
        
        url_layout.addLayout(input_layout)
        layout.addLayout(url_layout)
        
        # Progress bar (hidden by default)
        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 0)  # Indeterminate
        self.progress_bar.hide()
        layout.addWidget(self.progress_bar)
        
        # Release info frame (hidden until fetched)
        self.info_frame = QFrame()
        self.info_frame.setObjectName("infoFrame")
        self.info_frame.setStyleSheet("""
            QFrame#infoFrame {
                background-color: #2d2d2d;
                border-radius: 8px;
                border: 1px solid #3c3c3c;
                padding: 16px;
            }
        """)
        self.info_frame.hide()
        
        info_layout = QFormLayout(self.info_frame)
        info_layout.setSpacing(12)
        
        self.name_input = QLineEdit()
        info_layout.addRow("App Name:", self.name_input)
        
        self.version_label = QLabel()
        self.version_label.setStyleSheet("background: transparent;")
        info_layout.addRow("Latest Version:", self.version_label)
        
        self.package_combo = QComboBox()
        self.package_combo.currentIndexChanged.connect(self._on_package_selected)
        info_layout.addRow("Package Type:", self.package_combo)
        
        self.asset_combo = QComboBox()
        info_layout.addRow("Asset:", self.asset_combo)
        
        self.prerelease_check = QCheckBox("Include pre-releases")
        info_layout.addRow("", self.prerelease_check)
        
        self.auto_update_check = QCheckBox("Auto-update when available")
        info_layout.addRow("", self.auto_update_check)
        
        layout.addWidget(self.info_frame)
        
        # Buttons
        button_layout = QHBoxLayout()
        button_layout.addStretch()
        
        cancel_btn = QPushButton("Cancel")
        cancel_btn.setStyleSheet(get_button_style(COLOR_NEUTRAL))
        cancel_btn.clicked.connect(self.reject)
        button_layout.addWidget(cancel_btn)
        
        self.add_btn = QPushButton("Add App")
        self.add_btn.setEnabled(False)
        self.add_btn.setStyleSheet(get_button_style(COLOR_SUCCESS))
        self.add_btn.clicked.connect(self._add_app)
        button_layout.addWidget(self.add_btn)
        
        layout.addLayout(button_layout)
    
    def _fetch_release(self):
        url = self.url_input.text().strip()
        if not url:
            return
        
        try:
            owner, repo = self.github.parse_repo_url(url)
        except ValueError as e:
            QMessageBox.warning(self, "Invalid URL", str(e))
            return
        
        self.fetch_btn.setEnabled(False)
        self.progress_bar.show()
        
        # Run in thread
        self.thread = QThread()
        self.worker = FetchWorker(self.github, owner, repo)
        self.worker.moveToThread(self.thread)
        
        self.thread.started.connect(self.worker.run)
        self.worker.finished.connect(self._on_fetch_complete)
        self.worker.finished.connect(self.thread.quit)
        
        # Store owner/repo for later
        self._owner = owner
        self._repo = repo
        
        self.thread.start()
    
    def _on_fetch_complete(self, release: Release, error: str):
        self.fetch_btn.setEnabled(True)
        self.progress_bar.hide()
        
        if error:
            QMessageBox.warning(self, "Error", f"Failed to fetch release: {error}")
            return
        
        if not release:
            QMessageBox.warning(self, "No Releases", 
                               "This repository has no releases.")
            return
        
        self.release = release
        self._populate_release_info()
    
    def _populate_release_info(self):
        self.info_frame.show()
        
        # Set app name from repo name
        self.name_input.setText(self._repo)
        
        # Set version
        self.version_label.setText(self.release.version)
        
        # Populate package types
        self.package_combo.clear()
        package_types = set()
        for asset in self.release.assets:
            if asset.package_type:
                package_types.add(asset.package_type)
        
        # Sort with preferred type first
        preferred = self.installer.get_system_package_type()
        sorted_types = sorted(package_types, key=lambda t: (t != preferred, t))
        
        for pkg_type in sorted_types:
            self.package_combo.addItem(pkg_type.upper(), pkg_type)
        
        if not sorted_types:
            self.package_combo.addItem("Source (build from source)", "source")
        
        self.add_btn.setEnabled(True)
    
    def _on_package_selected(self, index):
        if index < 0:
            return
        
        pkg_type = self.package_combo.currentData()
        
        self.asset_combo.clear()
        for asset in self.release.assets:
            if asset.package_type == pkg_type:
                arch_str = f" ({asset.architecture})" if asset.architecture else ""
                self.asset_combo.addItem(f"{asset.name}{arch_str}", asset)
        
        # Select best match
        best = self.installer.select_best_asset(self.release.assets, pkg_type)
        if best:
            for i in range(self.asset_combo.count()):
                if self.asset_combo.itemData(i) == best:
                    self.asset_combo.setCurrentIndex(i)
                    break
    
    def _add_app(self):
        if not self.release:
            return
        
        asset = self.asset_combo.currentData()
        
        app_data = {
            'name': self.name_input.text().strip() or self._repo,
            'repo_url': f"https://github.com/{self._owner}/{self._repo}",
            'owner': self._owner,
            'repo': self._repo,
            'latest_version': self.release.version,
            'package_type': self.package_combo.currentData(),
            'asset': asset,
            'auto_update': self.auto_update_check.isChecked(),
            'include_prerelease': self.prerelease_check.isChecked(),
        }
        
        self.app_added.emit(app_data)
        self.accept()
