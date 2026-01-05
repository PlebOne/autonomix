import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'ui/home_screen.dart';
import 'services/database_service.dart';
import 'services/github_service.dart';
import 'services/installer_service.dart';

void main() {
  runApp(const AutonomixApp());
}

class AutonomixApp extends StatelessWidget {
  const AutonomixApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        Provider(create: (_) => DatabaseService()),
        Provider(create: (_) => GitHubService()),
        Provider(create: (_) => InstallerService()),
      ],
      child: MaterialApp(
        title: 'Autonomix',
        theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
          useMaterial3: true,
        ),
        home: const HomeScreen(),
      ),
    );
  }
}
