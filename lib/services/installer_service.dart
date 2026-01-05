import 'dart:io';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:http/http.dart' as http;
import '../models/install_type.dart';
import '../models/tracked_app.dart';

class InstallerService {
  Future<Directory> get _downloadsDir async {
    final dataDir = await getApplicationSupportDirectory();
    final dir = Directory(p.join(dataDir.path, 'downloads'));
    if (!await dir.exists()) {
      await dir.create(recursive: true);
    }
    return dir;
  }

  Future<Directory> get _appImageDir async {
    final dataDir = await getApplicationSupportDirectory();
    final dir = Directory(p.join(dataDir.path, 'appimages'));
    if (!await dir.exists()) {
      await dir.create(recursive: true);
    }
    return dir;
  }

  Future<InstallType?> detectSelfInstallType() async {
    // Check if installed via dpkg
    try {
      final result = await Process.run('dpkg', ['-s', 'autonomix']);
      if (result.exitCode == 0) return InstallType.deb;
    } catch (_) {}

    // Check if installed via rpm
    try {
      final result = await Process.run('rpm', ['-q', 'autonomix']);
      if (result.exitCode == 0) return InstallType.rpm;
    } catch (_) {}

    // Check if installed via flatpak
    try {
      final result = await Process.run('flatpak', ['info', 'io.github.plebone.autonomix']);
      if (result.exitCode == 0) return InstallType.flatpak;
    } catch (_) {}

    // Check if installed via snap
    try {
      final result = await Process.run('snap', ['info', 'autonomix']);
      if (result.exitCode == 0) return InstallType.snap;
    } catch (_) {}

    // Check if running from AppImage
    if (Platform.environment.containsKey('APPIMAGE')) {
      return InstallType.appImage;
    }

    // Check if binary in ~/.local/bin
    // This is harder to check reliably in Dart without more context, 
    // but we can check the executable path.
    final exePath = Platform.resolvedExecutable;
    final home = Platform.environment['HOME'];
    if (home != null && exePath.startsWith('$home/.local/bin')) {
      return InstallType.binary;
    }

    return null;
  }

  Future<File> downloadFile(String url, String filename) async {
    final dir = await _downloadsDir;
    final file = File(p.join(dir.path, filename));
    
    final response = await http.get(Uri.parse(url));
    if (response.statusCode == 200) {
      await file.writeAsBytes(response.bodyBytes);
      return file;
    } else {
      throw Exception('Failed to download file: ${response.statusCode}');
    }
  }

  Future<({String? launchCommand, String? packageName})> installPackage(File file, InstallType type) async {
    switch (type) {
      case InstallType.deb:
        String? pkgName;
        try {
          final res = await Process.run('dpkg-deb', ['-f', file.path, 'Package']);
          if (res.exitCode == 0) pkgName = res.stdout.toString().trim();
        } catch (_) {}

        await _runPrivileged('dpkg', ['-i', file.path]);
        return (launchCommand: null, packageName: pkgName);

      case InstallType.rpm:
        String? pkgName;
        try {
          final res = await Process.run('rpm', ['-qp', '--queryformat', '%{NAME}', file.path]);
          if (res.exitCode == 0) pkgName = res.stdout.toString().trim();
        } catch (_) {}

        await _runPrivileged('rpm', ['-i', file.path]);
        return (launchCommand: null, packageName: pkgName);

      case InstallType.flatpak:
        await Process.run('flatpak', ['install', '-y', file.path]);
        return (launchCommand: null, packageName: null);

      case InstallType.appImage:
        final appImageDir = await _appImageDir;
        final target = File(p.join(appImageDir.path, p.basename(file.path)));
        await file.copy(target.path);
        await Process.run('chmod', ['+x', target.path]);
        return (launchCommand: target.path, packageName: null);

      default:
        throw Exception('Installation not supported for ${type.name}');
    }
  }

  Future<void> uninstallPackage(TrackedApp app) async {
    if (app.installType == InstallType.appImage && app.launchCommand != null) {
       final file = File(app.launchCommand!);
       if (await file.exists()) await file.delete();
    } else if (app.installType == InstallType.deb && app.packageName != null) {
       await _runPrivileged('dpkg', ['-r', app.packageName!]);
    } else if (app.installType == InstallType.rpm && app.packageName != null) {
       await _runPrivileged('rpm', ['-e', app.packageName!]);
    } else {
      throw Exception('Uninstall not supported for this app (missing package info)');
    }
  }

  Future<void> launchApp(TrackedApp app) async {
    if (app.launchCommand != null) {
      // If we have a stored command/path, use it
      if (app.installType == InstallType.appImage) {
        await Process.start(app.launchCommand!, []);
      } else {
        // For others, it might be a command in PATH
        await Process.start(app.launchCommand!, []);
      }
      return;
    }

    if (app.installType == InstallType.appImage && app.installedVersion != null) {
       // Fallback for old AppImages without stored path
       final appImageDir = await _appImageDir;
       await for (final entity in appImageDir.list()) {
         if (entity is File && entity.path.toLowerCase().contains(app.repoName.toLowerCase())) {
           await Process.start(entity.path, []);
           return;
         }
       }
       throw Exception('Could not find AppImage to launch');
    } else {
      // For system installs, try running the repo name as command
      try {
        await Process.start(app.repoName, []);
      } catch (e) {
        // Try lowercase as fallback (common for Linux binaries)
        if (app.repoName != app.repoName.toLowerCase()) {
          try {
            await Process.start(app.repoName.toLowerCase(), []);
            return;
          } catch (_) {
            // Ignore and throw original error
          }
        }
        throw Exception('Could not launch ${app.repoName}: $e');
      }
    }
  }

  InstallType? identifyAssetType(String filename) {
    final lower = filename.toLowerCase();
    if (lower.endsWith('.deb')) return InstallType.deb;
    if (lower.endsWith('.rpm')) return InstallType.rpm;
    if (lower.endsWith('.appimage')) return InstallType.appImage;
    if (lower.endsWith('.flatpak')) return InstallType.flatpak;
    if (lower.endsWith('.snap')) return InstallType.snap;
    return null;
  }

  Future<void> _runPrivileged(String command, List<String> args) async {
    // Try pkexec first
    try {
      final result = await Process.run('pkexec', [command, ...args]);
      if (result.exitCode != 0) {
        throw Exception('Command failed: ${result.stderr}');
      }
    } catch (e) {
      throw Exception('Failed to run privileged command: $e');
    }
  }
}
