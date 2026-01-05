import 'package:flutter_test/flutter_test.dart';
import 'package:autonomix/models/tracked_app.dart';
import 'package:autonomix/models/install_type.dart';

void main() {
  group('TrackedApp', () {
    test('hasUpdate returns true when latest version is newer', () {
      final app = TrackedApp(
        repoOwner: 'owner',
        repoName: 'repo',
        displayName: 'App',
        installedVersion: '1.0.0',
        latestVersion: '1.0.1',
        createdAt: DateTime.now(),
      );

      expect(app.hasUpdate, isTrue);
    });

    test('hasUpdate returns false when versions are equal', () {
      final app = TrackedApp(
        repoOwner: 'owner',
        repoName: 'repo',
        displayName: 'App',
        installedVersion: '1.0.0',
        latestVersion: '1.0.0',
        createdAt: DateTime.now(),
      );

      expect(app.hasUpdate, isFalse);
    });

    test('hasUpdate returns false when installed version is newer', () {
      final app = TrackedApp(
        repoOwner: 'owner',
        repoName: 'repo',
        displayName: 'App',
        installedVersion: '1.0.1',
        latestVersion: '1.0.0',
        createdAt: DateTime.now(),
      );

      expect(app.hasUpdate, isFalse);
    });

    test('hasUpdate handles v-prefix correctly', () {
      final app = TrackedApp(
        repoOwner: 'owner',
        repoName: 'repo',
        displayName: 'App',
        installedVersion: 'v1.0.0',
        latestVersion: '1.0.1',
        createdAt: DateTime.now(),
      );

      expect(app.hasUpdate, isTrue);
    });

    test('hasUpdate returns false if installedVersion is null', () {
      final app = TrackedApp(
        repoOwner: 'owner',
        repoName: 'repo',
        displayName: 'App',
        installedVersion: null,
        latestVersion: '1.0.0',
        createdAt: DateTime.now(),
      );

      expect(app.hasUpdate, isFalse);
    });

    test('repoUrl is correct', () {
      final app = TrackedApp(
        repoOwner: 'owner',
        repoName: 'repo',
        displayName: 'App',
        createdAt: DateTime.now(),
      );

      expect(app.repoUrl, 'https://github.com/owner/repo');
    });
  });
}
