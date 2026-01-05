import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/tracked_app.dart';
import '../models/install_type.dart';
import '../services/database_service.dart';
import '../services/github_service.dart';
import '../services/installer_service.dart';
import 'widgets/app_list_item.dart';
import 'widgets/add_app_dialog.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  List<TrackedApp> _apps = [];
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadApps();
  }

  Future<void> _loadApps() async {
    setState(() => _isLoading = true);
    try {
      final db = context.read<DatabaseService>();
      final apps = await db.getAllApps();
      setState(() {
        _apps = apps;
        _isLoading = false;
      });
    } catch (e) {
      setState(() => _isLoading = false);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Error loading apps: $e')),
        );
      }
    }
  }

  Future<void> _addApp() async {
    final result = await showDialog<Map<String, String>>(
      context: context,
      builder: (context) => const AddAppDialog(),
    );

    if (result != null) {
      try {
        await context.read<DatabaseService>().addApp(
          result['owner']!,
          result['repo']!,
          result['name']!,
        );
        _loadApps();
      } catch (e) {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('Error adding app: $e')),
          );
        }
      }
    }
  }

  Future<void> _checkForUpdates() async {
    final gh = context.read<GitHubService>();
    final db = context.read<DatabaseService>();
    
    for (var app in _apps) {
      try {
        final release = await gh.getLatestRelease(app.repoOwner, app.repoName);
        final updatedApp = app.copyWith(
          latestVersion: release.tagName,
          lastChecked: DateTime.now(),
        );
        await db.updateApp(updatedApp);
      } catch (e) {
        print('Error checking updates for ${app.displayName}: $e');
      }
    }
    _loadApps();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Autonomix'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _checkForUpdates,
            tooltip: 'Check for updates',
          ),
        ],
      ),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _apps.isEmpty
              ? const Center(child: Text('No apps tracked. Add one!'))
              : ListView.builder(
                  itemCount: _apps.length,
                  itemBuilder: (context, index) {
                    return AppListItem(
                      app: _apps[index],
                      onTap: () => _showAppDetails(_apps[index]),
                    );
                  },
                ),
      floatingActionButton: FloatingActionButton(
        onPressed: _addApp,
        child: const Icon(Icons.add),
      ),
    );
  }

  Future<void> _showAppDetails(TrackedApp app) async {
    await showModalBottomSheet(
      context: context,
      builder: (context) => AppDetailsSheet(app: app),
    );
    _loadApps();
  }
}

class AppDetailsSheet extends StatefulWidget {
  final TrackedApp app;

  const AppDetailsSheet({super.key, required this.app});

  @override
  State<AppDetailsSheet> createState() => _AppDetailsSheetState();
}

class _AppDetailsSheetState extends State<AppDetailsSheet> {
  bool _isInstalling = false;
  String? _statusMessage;

  Future<void> _install(BuildContext context) async {
    setState(() {
      _isInstalling = true;
      _statusMessage = 'Fetching releases...';
    });

    try {
      final gh = context.read<GitHubService>();
      final installer = context.read<InstallerService>();
      final db = context.read<DatabaseService>();

      final release = await gh.getLatestRelease(widget.app.repoOwner, widget.app.repoName);
      
      // Find candidates
      final candidates = <InstallType, dynamic>{}; // dynamic to avoid importing ReleaseAsset
      for (var asset in release.assets) {
        final type = installer.identifyAssetType(asset.name);
        if (type != null) {
          candidates[type] = asset;
        }
      }

      if (candidates.isEmpty) {
        throw Exception('No supported assets found in release');
      }

      if (!mounted) return;

      // Show selection dialog
      final selectedType = await showDialog<InstallType>(
        context: context,
        builder: (context) => SimpleDialog(
          title: const Text('Select Package Type'),
          children: candidates.keys.map((type) {
            return SimpleDialogOption(
              onPressed: () => Navigator.pop(context, type),
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: Row(
                  children: [
                    Icon(_getIconForType(type)),
                    const SizedBox(width: 12),
                    Text(type.displayName),
                  ],
                ),
              ),
            );
          }).toList(),
        ),
      );

      if (selectedType == null) {
        setState(() => _isInstalling = false);
        return;
      }

      final asset = candidates[selectedType]!;
      
      setState(() => _statusMessage = 'Downloading ${asset.name}...');
      final file = await installer.downloadFile(asset.browserDownloadUrl, asset.name);

      setState(() => _statusMessage = 'Installing...');
      final result = await installer.installPackage(file, selectedType);

      // Update DB
      final updatedApp = widget.app.copyWith(
        installedVersion: release.tagName,
        installType: selectedType,
        launchCommand: result.launchCommand,
        packageName: result.packageName,
        lastChecked: DateTime.now(),
      );
      await db.updateApp(updatedApp);

      if (mounted) {
        Navigator.pop(context); // Close sheet
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Installation successful')),
        );
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isInstalling = false;
          _statusMessage = 'Error: $e';
        });
      }
    }
  }

  IconData _getIconForType(InstallType type) {
    switch (type) {
      case InstallType.deb: return Icons.grid_view;
      case InstallType.rpm: return Icons.settings;
      case InstallType.appImage: return Icons.extension;
      case InstallType.flatpak: return Icons.layers;
      case InstallType.snap: return Icons.shopping_bag;
      default: return Icons.download;
    }
  }

  Future<void> _uninstall(BuildContext context) async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Uninstall App'),
        content: const Text('Are you sure you want to uninstall this app?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Uninstall'),
          ),
        ],
      ),
    );

    if (confirm != true) return;

    setState(() {
      _isInstalling = true;
      _statusMessage = 'Uninstalling...';
    });

    try {
      await context.read<InstallerService>().uninstallPackage(widget.app);

      // Update DB - Clear installed fields
      final updatedApp = TrackedApp(
        id: widget.app.id,
        repoOwner: widget.app.repoOwner,
        repoName: widget.app.repoName,
        displayName: widget.app.displayName,
        createdAt: widget.app.createdAt,
        latestVersion: widget.app.latestVersion,
        lastChecked: widget.app.lastChecked,
        // Cleared:
        installedVersion: null,
        installType: null,
        launchCommand: null,
        packageName: null,
      );
      
      await context.read<DatabaseService>().updateApp(updatedApp);

      if (mounted) {
        Navigator.pop(context); // Close sheet
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Uninstallation successful')),
        );
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isInstalling = false;
        });
        showDialog(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('Uninstall Error'),
            content: Text(e.toString()),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('OK'),
              ),
            ],
          ),
        );
      }
    }
  }

  Future<void> _launch(BuildContext context) async {
    try {
      await context.read<InstallerService>().launchApp(widget.app);
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (mounted) {
        showDialog(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('Launch Error'),
            content: Text(e.toString()),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('OK'),
              ),
            ],
          ),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(widget.app.displayName, style: Theme.of(context).textTheme.headlineSmall),
          const SizedBox(height: 8),
          Text('Repo: ${widget.app.repoOwner}/${widget.app.repoName}'),
          Text('Installed: ${widget.app.installedVersion ?? "Not installed"}'),
          Text('Latest: ${widget.app.latestVersion ?? "Unknown"}'),
          const SizedBox(height: 16),
          if (_isInstalling) ...[
            const LinearProgressIndicator(),
            const SizedBox(height: 8),
            Text(_statusMessage ?? ''),
          ] else
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                if (widget.app.isInstalled) ...[
                  OutlinedButton.icon(
                    onPressed: () => _uninstall(context),
                    icon: const Icon(Icons.delete),
                    label: const Text('Uninstall'),
                  ),
                  const SizedBox(width: 8),
                  FilledButton.icon(
                    onPressed: () => _launch(context),
                    icon: const Icon(Icons.play_arrow),
                    label: const Text('Launch'),
                  ),
                ],
                const SizedBox(width: 8),
                if (widget.app.hasUpdate)
                  FilledButton.icon(
                    onPressed: () => _install(context),
                    icon: const Icon(Icons.system_update),
                    label: const Text('Update'),
                  ),
                const SizedBox(width: 8),
                if (!widget.app.isInstalled)
                  FilledButton.icon(
                    onPressed: () => _install(context),
                    icon: const Icon(Icons.download),
                    label: const Text('Install'),
                  ),
              ],
            ),
        ],
      ),
    );
  }
}
