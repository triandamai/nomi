import 'dart:convert';
import 'package:drift/drift.dart';
import 'package:nomi_mobile/data/models/conversation.dart' as model;
import 'package:nomi_mobile/data/models/message.dart' as msg_model;
import 'package:nomi_mobile/core/api/api_client.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;

class ChatRepository {
  final ApiClient _apiClient;
  final db.NomiDatabase _db;

  ChatRepository(this._apiClient, this._db);

  // 📡 Watchers (SSoT from Database)
  Stream<List<db.Conversation>> watchConversations() => _db.watchConversations();
  Stream<List<db.Message>> watchMessages(String conversationId) => _db.watchMessages(conversationId);

  // 🔄 Sync: Remote to Local
  Future<void> syncConversations() async {
    final response = await _apiClient.dio.get('/conversations');
    if (response.statusCode == 200) {
      final List<dynamic> jsonList = response.data['data'] ?? [];
      final List<db.ConversationsCompanion> companions = jsonList.map((e) {
        final conv = model.Conversation.fromJson(e);
        return db.ConversationsCompanion.insert(
          id: conv.id,
          name: Value(conv.name),
          cumulativeTokens: Value(conv.cumulativeTokens),
          maxTokenUsage: Value(conv.maxTokenUsage),
          createdAt: DateTime.parse(conv.createdAt),
          updatedAt: DateTime.parse(conv.updatedAt),
        );
      }).toList();
      await _db.upsertConversations(companions);
    }
  }

  Future<void> syncMessages(String conversationId, {String? cursor}) async {
    final Map<String, dynamic> query = {'limit': 50};
    if (cursor != null) query['cursor'] = cursor;

    final response = await _apiClient.dio.get(
      '/conversations/$conversationId/messages',
      queryParameters: query,
    );

    if (response.statusCode == 200) {
      final List<dynamic> msgList = response.data['data']['messages'] ?? [];
      final List<db.MessagesCompanion> companions = msgList.map((e) {
        final m = msg_model.Message.fromJson(e);
        return db.MessagesCompanion.insert(
          id: m.id,
          conversationId: m.conversationId,
          role: m.role,
          content: m.content,
          displayName: Value(m.displayName),
          thought: Value(m.thought),
          imageUrl: Value(m.imageUrl),
          videoUrl: Value(m.video_url),
          audioUrl: Value(m.audio_url),
          documentUrl: Value(m.document_url),
          stickerUrl: Value(m.sticker_url),
          userId: Value(m.userId),
          totalTokens: Value(m.totalTokens),
          createdAt: DateTime.parse(m.createdAt),
          metadata: Value(m.metadata != null ? jsonEncode(m.metadata) : null),
          replyToId: Value(m.replyToId),
          repliedMessage: Value(m.repliedMessage != null ? jsonEncode(m.repliedMessage!.toJson()) : null),
          syncStatus: db.SyncStatus.synced,
        );
      }).toList();
      await _db.upsertMessages(companions);
    }
  }

  Future<void> sendChatMessage(
    String conversationId,
    String content, {
    String? replyToId,
    msg_model.RepliedMessage? repliedMessage,
    Map<String, String>? media,
  }) async {
    // 💡 Offline-First: Write to local DB first (Optimistic UI)
    final String tempId = 'temp_${DateTime.now().millisecondsSinceEpoch}';
    final companion = db.MessagesCompanion.insert(
      id: tempId,
      conversationId: conversationId,
      role: 'user',
      content: content,
      createdAt: DateTime.now(),
      replyToId: Value(replyToId),
      repliedMessage: Value(repliedMessage != null ? jsonEncode(repliedMessage.toJson()) : null),
      syncStatus: db.SyncStatus.pending,
    );
    await _db.upsertMessages([companion]);

    // 📡 Background attempt to sync
    final Map<String, dynamic> data = {
      'conversation_id': conversationId,
      'message': content,
      if (replyToId != null) 'reply_to_id': replyToId,
      ...?media,
    };
    
    try {
      final response = await _apiClient.dio.post('/chat/stream', data: data);
      if (response.statusCode! >= 200 && response.statusCode! < 300) {
        await _db.customStatement('DELETE FROM messages WHERE id = ?', [tempId]);
      }
    } catch (e) {
      // Failed: Update status to FAILED
      await _db.customStatement('UPDATE messages SET sync_status = ? WHERE id = ?', [db.SyncStatus.failed.index, tempId]);
    }
  }
}
