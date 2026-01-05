import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../services/github_service.dart';

class AddAppDialog extends StatefulWidget {
  const AddAppDialog({super.key});

  @override
  State<AddAppDialog> createState() => _AddAppDialogState();
}

class _AddAppDialogState extends State<AddAppDialog> {
  final _formKey = GlobalKey<FormState>();
  final _urlController = TextEditingController();
  final _ownerController = TextEditingController();
  final _repoController = TextEditingController();
  final _nameController = TextEditingController();

  bool _isFetching = false;
  bool _hasFetched = false;
  String? _error;

  Future<void> _fetchDetails() async {
    final url = _urlController.text.trim();
    if (url.isEmpty) return;

    setState(() {
      _isFetching = true;
      _error = null;
    });

    try {
      // Parse URL
      // Supports:
      // https://github.com/owner/repo
      // https://github.com/owner/repo/releases...
      final uri = Uri.parse(url);
      if (uri.host != 'github.com') {
        throw Exception('Not a GitHub URL');
      }

      final segments = uri.pathSegments;
      if (segments.length < 2) {
        throw Exception('Invalid repository URL');
      }

      final owner = segments[0];
      final repo = segments[1];

      // Fetch details
      final gh = context.read<GitHubService>();
      final info = await gh.getRepository(owner, repo);

      setState(() {
        _ownerController.text = info['owner']['login'];
        _repoController.text = info['name'];
        _nameController.text = info['description'] ?? info['name']; // Use description or name
        _hasFetched = true;
        _isFetching = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _isFetching = false;
      });
    }
  }

  @override
  void dispose() {
    _urlController.dispose();
    _ownerController.dispose();
    _repoController.dispose();
    _nameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Add App'),
      content: SizedBox(
        width: 400,
        child: Form(
          key: _formKey,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              if (!_hasFetched) ...[
                TextFormField(
                  controller: _urlController,
                  decoration: InputDecoration(
                    labelText: 'GitHub URL',
                    hintText: 'https://github.com/owner/repo',
                    suffixIcon: IconButton(
                      icon: const Icon(Icons.paste),
                      onPressed: () async {
                        // TODO: Paste from clipboard
                      },
                    ),
                  ),
                  onFieldSubmitted: (_) => _fetchDetails(),
                ),
                const SizedBox(height: 16),
                if (_isFetching)
                  const Center(child: CircularProgressIndicator())
                else
                  FilledButton(
                    onPressed: _fetchDetails,
                    child: const Text('Fetch Details'),
                  ),
              ],
              if (_hasFetched) ...[
                TextFormField(
                  controller: _ownerController,
                  decoration: const InputDecoration(labelText: 'Repo Owner'),
                  validator: (v) => v?.isEmpty == true ? 'Required' : null,
                ),
                TextFormField(
                  controller: _repoController,
                  decoration: const InputDecoration(labelText: 'Repo Name'),
                  validator: (v) => v?.isEmpty == true ? 'Required' : null,
                ),
                TextFormField(
                  controller: _nameController,
                  decoration: const InputDecoration(labelText: 'Display Name'),
                  validator: (v) => v?.isEmpty == true ? 'Required' : null,
                ),
              ],
              if (_error != null) ...[
                const SizedBox(height: 16),
                Text(
                  _error!,
                  style: TextStyle(color: Theme.of(context).colorScheme.error),
                ),
              ],
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        if (_hasFetched)
          FilledButton(
            onPressed: () {
              if (_formKey.currentState!.validate()) {
                Navigator.pop(context, {
                  'owner': _ownerController.text,
                  'repo': _repoController.text,
                  'name': _nameController.text,
                });
              }
            },
            child: const Text('Add'),
          ),
      ],
    );
  }
}
