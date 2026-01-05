import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:autonomix/ui/home_screen.dart';
import 'package:autonomix/services/database_service.dart';
import 'package:autonomix/services/github_service.dart';
import 'package:autonomix/services/installer_service.dart';
import 'package:autonomix/models/tracked_app.dart';

class MockDatabaseService extends DatabaseService {
  @override
  Future<List<TrackedApp>> getAllApps() async {
    return [
      TrackedApp(
        id: 1,
        repoOwner: 'owner',
        repoName: 'repo',
        displayName: 'Test App',
        installedVersion: '1.0.0',
        latestVersion: '1.0.1',
        createdAt: DateTime.now(),
      ),
    ];
  }

  @override
  Future<int> addApp(String repoOwner, String repoName, String displayName) async {
    return 1;
  }
}

class MockGitHubService extends GitHubService {}
class MockInstallerService extends InstallerService {}

void main() {
  testWidgets('HomeScreen shows apps', (WidgetTester tester) async {
    await tester.pumpWidget(
      MultiProvider(
        providers: [
          Provider<DatabaseService>(create: (_) => MockDatabaseService()),
          Provider<GitHubService>(create: (_) => MockGitHubService()),
          Provider<InstallerService>(create: (_) => MockInstallerService()),
        ],
        child: const MaterialApp(
          home: HomeScreen(),
        ),
      ),
    );

    // Initial load
    await tester.pump(); 
    // Wait for future to complete
    await tester.pump(const Duration(milliseconds: 100));

    expect(find.text('Test App'), findsOneWidget);
    expect(find.text('owner/repo'), findsOneWidget);
    expect(find.text('Update Available'), findsOneWidget);
  });

  testWidgets('Add App dialog opens', (WidgetTester tester) async {
    await tester.pumpWidget(
      MultiProvider(
        providers: [
          Provider<DatabaseService>(create: (_) => MockDatabaseService()),
          Provider<GitHubService>(create: (_) => MockGitHubService()),
          Provider<InstallerService>(create: (_) => MockInstallerService()),
        ],
        child: const MaterialApp(
          home: HomeScreen(),
        ),
      ),
    );

    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    await tester.tap(find.byIcon(Icons.add));
    await tester.pumpAndSettle();

    expect(find.text('Add App'), findsOneWidget);
    expect(find.text('GitHub URL'), findsOneWidget);
  });
}
