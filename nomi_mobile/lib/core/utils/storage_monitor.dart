import 'dart:io';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as p;

class StorageMonitor {
  static Future<Map<String, dynamic>> getStorageMetrics() async {
    final dbFolder = await getApplicationDocumentsDirectory();
    final dbFile = File(p.join(dbFolder.path, 'nomi.sqlite'));
    
    int dbSize = 0;
    if (await dbFile.exists()) {
      dbSize = await dbFile.length();
    }

    final cacheDir = await getTemporaryDirectory();
    int cacheSize = await _getDirectorySize(cacheDir);

    return {
      'dbPath': dbFile.path,
      'dbSize': dbSize,
      'cacheSize': cacheSize,
      'dbExists': await dbFile.exists(),
    };
  }

  static Future<int> _getDirectorySize(Directory dir) async {
    int size = 0;
    if (await dir.exists()) {
      await for (final entity in dir.list(recursive: true)) {
        if (entity is File) {
          size += await entity.length();
        }
      }
    }
    return size;
  }
}
