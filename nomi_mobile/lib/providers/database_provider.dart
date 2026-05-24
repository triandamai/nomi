import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nomi_mobile/core/db/database.dart';

final databaseProvider = Provider<NomiDatabase>((ref) {
  final db = NomiDatabase();
  
  // Close database when provider is disposed
  ref.onDispose(() => db.close());
  
  return db;
});
