import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nomi_mobile/core/api/api_client.dart';
import 'package:nomi_mobile/data/repositories/auth_repository.dart';
import 'package:nomi_mobile/data/repositories/chat_repository.dart';
import 'package:nomi_mobile/data/repositories/file_repository.dart';
import 'package:nomi_mobile/providers/database_provider.dart';

final apiClientProvider = Provider<ApiClient>((ref) {
  return ApiClient();
});

final authRepositoryProvider = Provider<AuthRepository>((ref) {
  return AuthRepository(ref.watch(apiClientProvider));
});

final chatRepositoryProvider = Provider<ChatRepository>((ref) {
  return ChatRepository(
    ref.watch(apiClientProvider),
    ref.watch(databaseProvider),
  );
});

final fileRepositoryProvider = Provider<FileRepository>((ref) {
  return FileRepository(ref.watch(apiClientProvider));
});
