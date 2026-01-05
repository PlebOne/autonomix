import 'install_type.dart';

class TrackedApp {
  final int? id;
  final String repoOwner;
  final String repoName;
  final String displayName;
  final String? installedVersion;
  final String? latestVersion;
  final InstallType? installType;
  final String? launchCommand;
  final String? packageName;
  final DateTime? lastChecked;
  final DateTime createdAt;

  TrackedApp({
    this.id,
    required this.repoOwner,
    required this.repoName,
    required this.displayName,
    this.installedVersion,
    this.latestVersion,
    this.installType,
    this.launchCommand,
    this.packageName,
    this.lastChecked,
    required this.createdAt,
  });

  String get repoUrl => 'https://github.com/$repoOwner/$repoName';

  bool get hasUpdate {
    if (installedVersion == null || latestVersion == null) return false;
    
    final installed = _normalizeVersion(installedVersion!);
    final latest = _normalizeVersion(latestVersion!);
    
    if (installed == latest) return false;
    
    return _isNewerVersion(latest, installed);
  }

  bool get isInstalled => installedVersion != null;

  static String _normalizeVersion(String version) {
    var v = version.trim();
    if (v.startsWith('v') || v.startsWith('V')) {
      v = v.substring(1);
    }
    return v.toLowerCase();
  }

  static bool _isNewerVersion(String newVersion, String oldVersion) {
    // Simple string comparison for now, mimicking the fallback in Rust
    // A real implementation would use a semver library
    return newVersion.compareTo(oldVersion) > 0;
  }

  Map<String, dynamic> toMap() {
    return {
      'id': id,
      'repo_owner': repoOwner,
      'repo_name': repoName,
      'display_name': displayName,
      'installed_version': installedVersion,
      'latest_version': latestVersion,
      'install_type': installType?.name,
      'launch_command': launchCommand,
      'package_name': packageName,
      'last_checked': lastChecked?.toIso8601String(),
      'created_at': createdAt.toIso8601String(),
    };
  }

  factory TrackedApp.fromMap(Map<String, dynamic> map) {
    return TrackedApp(
      id: map['id'] as int?,
      repoOwner: map['repo_owner'] as String,
      repoName: map['repo_name'] as String,
      displayName: map['display_name'] as String,
      installedVersion: map['installed_version'] as String?,
      latestVersion: map['latest_version'] as String?,
      installType: InstallType.fromString(map['install_type'] as String?),
      launchCommand: map['launch_command'] as String?,
      packageName: map['package_name'] as String?,
      lastChecked: map['last_checked'] != null 
          ? DateTime.parse(map['last_checked'] as String) 
          : null,
      createdAt: DateTime.parse(map['created_at'] as String),
    );
  }
  
  TrackedApp copyWith({
    int? id,
    String? repoOwner,
    String? repoName,
    String? displayName,
    String? installedVersion,
    String? latestVersion,
    InstallType? installType,
    String? launchCommand,
    String? packageName,
    DateTime? lastChecked,
    DateTime? createdAt,
  }) {
    return TrackedApp(
      id: id ?? this.id,
      repoOwner: repoOwner ?? this.repoOwner,
      repoName: repoName ?? this.repoName,
      displayName: displayName ?? this.displayName,
      installedVersion: installedVersion ?? this.installedVersion,
      latestVersion: latestVersion ?? this.latestVersion,
      installType: installType ?? this.installType,
      launchCommand: launchCommand ?? this.launchCommand,
      packageName: packageName ?? this.packageName,
      lastChecked: lastChecked ?? this.lastChecked,
      createdAt: createdAt ?? this.createdAt,
    );
  }
}
