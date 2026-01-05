import 'dart:convert';
import 'package:http/http.dart' as http;
import '../models/release.dart';

class GitHubService {
  static const String _baseUrl = 'https://api.github.com';
  static const String _userAgent = 'Autonomix/0.3.4';

  Future<Release> getLatestRelease(String owner, String repo) async {
    final url = Uri.parse('$_baseUrl/repos/$owner/$repo/releases/latest');
    
    final response = await http.get(
      url,
      headers: {'User-Agent': _userAgent},
    );

    if (response.statusCode == 200) {
      return Release.fromJson(jsonDecode(response.body));
    } else {
      throw Exception('Failed to load latest release: ${response.statusCode}');
    }
  }

  Future<List<Release>> getReleases(String owner, String repo) async {
    final url = Uri.parse('$_baseUrl/repos/$owner/$repo/releases?per_page=10');
    
    final response = await http.get(
      url,
      headers: {'User-Agent': _userAgent},
    );

    if (response.statusCode == 200) {
      final List<dynamic> list = jsonDecode(response.body);
      return list.map((e) => Release.fromJson(e)).toList();
    } else {
      throw Exception('Failed to load releases: ${response.statusCode}');
    }
  }

  Future<Map<String, dynamic>> getRepository(String owner, String repo) async {
    final url = Uri.parse('$_baseUrl/repos/$owner/$repo');
    
    final response = await http.get(
      url,
      headers: {'User-Agent': _userAgent},
    );

    if (response.statusCode == 200) {
      return jsonDecode(response.body) as Map<String, dynamic>;
    } else {
      throw Exception('Failed to load repository: ${response.statusCode}');
    }
  }
}
