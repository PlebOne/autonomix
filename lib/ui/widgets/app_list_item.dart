import 'package:flutter/material.dart';
import '../../models/tracked_app.dart';

class AppListItem extends StatelessWidget {
  final TrackedApp app;
  final VoidCallback onTap;

  const AppListItem({
    super.key,
    required this.app,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return ListTile(
      title: Text(app.displayName),
      subtitle: Text('${app.repoOwner}/${app.repoName}'),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (app.hasUpdate)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: Colors.orange.shade100,
                borderRadius: BorderRadius.circular(12),
              ),
              child: Text(
                'Update Available',
                style: TextStyle(color: Colors.orange.shade900, fontSize: 12),
              ),
            ),
          const SizedBox(width: 8),
          if (app.isInstalled)
            const Icon(Icons.check_circle, color: Colors.green)
          else
            const Icon(Icons.circle_outlined, color: Colors.grey),
        ],
      ),
      onTap: onTap,
    );
  }
}
