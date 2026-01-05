import 'dart:convert';
import 'dart:io';
import 'package:path/path.dart';
import 'package:path_provider/path_provider.dart';
import '../models/tracked_app.dart';

class DatabaseService {
  File? _file;

  DatabaseService();

  Future<File> get _dbFile async {
    if (_file != null) return _file!;
    final configDir = await getApplicationSupportDirectory();
    await Directory(configDir.path).create(recursive: true);
    _file = File(join(configDir.path, 'apps.json'));
    return _file!;
  }

  Future<List<TrackedApp>> getAllApps() async {
    final file = await _dbFile;
    if (!await file.exists()) return [];

    try {
      final content = await file.readAsString();
      if (content.isEmpty) return [];
      
      final List<dynamic> jsonList = jsonDecode(content);
      return jsonList.map((e) => TrackedApp.fromMap(e)).toList()
        ..sort((a, b) => a.displayName.compareTo(b.displayName));
    } catch (e) {
      print('Error reading DB: $e');
      return [];
    }
  }

  Future<void> _saveApps(List<TrackedApp> apps) async {
    final file = await _dbFile;
    final jsonList = apps.map((e) => e.toMap()).toList();
    await file.writeAsString(jsonEncode(jsonList));
  }

  Future<int> addApp(String repoOwner, String repoName, String displayName) async {
    final apps = await getAllApps();
    
    // Check for duplicates
    if (apps.any((a) => a.repoOwner == repoOwner && a.repoName == repoName)) {
      throw Exception('App already exists');
    }

    // Generate ID
    final id = (apps.isEmpty ? 0 : apps.map((e) => e.id ?? 0).reduce((a, b) => a > b ? a : b)) + 1;

    final newApp = TrackedApp(
      id: id,
      repoOwner: repoOwner,
      repoName: repoName,
      displayName: displayName,
      createdAt: DateTime.now(),
    );

    apps.add(newApp);
    await _saveApps(apps);
    return id;
  }

  Future<void> updateApp(TrackedApp app) async {
    final apps = await getAllApps();
    final index = apps.indexWhere((a) => a.id == app.id);
    
    if (index != -1) {
      apps[index] = app;
      await _saveApps(apps);
    }
  }

  Future<void> deleteApp(int id) async {
    final apps = await getAllApps();
    apps.removeWhere((a) => a.id == id);
    await _saveApps(apps);
  }
}
