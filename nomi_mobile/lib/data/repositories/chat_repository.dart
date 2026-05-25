import 'dart:convert';
import 'package:dio/dio.dart';
import 'package:drift/drift.dart';
import 'package:nomi_mobile/data/models/conversation.dart' as conv_model;
import 'package:nomi_mobile/data/models/message.dart' as msg_model;
import 'package:nomi_mobile/data/models/reminder.dart' as rem_model;
import 'package:nomi_mobile/data/models/transaction.dart' as tx_model;
import 'package:nomi_mobile/data/models/health_metric.dart' as health_model;
import 'package:nomi_mobile/data/models/plugin.dart' as plugin_model;
import 'package:nomi_mobile/data/models/srp_proposal.dart' as srp_model;
import 'package:nomi_mobile/data/models/user_profile.dart' as user_model;
import 'package:nomi_mobile/data/models/user_detail.dart' as user_detail_model;
import 'package:nomi_mobile/data/models/storage_item.dart' as storage_model;
import 'package:nomi_mobile/data/models/reinforcement_state.dart' as reinforcement_model;
import 'package:nomi_mobile/data/models/admin_conversation.dart' as admin_conv_model;
import 'package:nomi_mobile/data/models/guardrail_pattern.dart' as guardrail_model;
import 'package:nomi_mobile/data/models/skill.dart' as skill_model;
import 'package:nomi_mobile/core/api/api_client.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;

class ChatRepository {
  final ApiClient _apiClient;
  final db.NomiDatabase _db;

  ChatRepository(this._apiClient, this._db);

  // 📡 Watchers (SSoT from Database)
  Stream<List<db.Conversation>> watchConversations() => _db.watchConversations();
  Stream<List<db.Message>> watchMessages(String conversationId) => _db.watchMessages(conversationId);
  Stream<List<db.Reminder>> watchReminders() => _db.watchReminders();
  Stream<List<db.Transaction>> watchTransactions({String? category, String? search}) => 
    _db.watchTransactions(category: category, search: search);
  Stream<List<db.HealthMetric>> watchHealthHistory({DateTime? start, DateTime? end}) => 
    _db.watchHealthHistory(start: start, end: end);
  Stream<List<db.Plugin>> watchPlugins({String? search}) => _db.watchPlugins(search: search);

  // 🛡️ Helper: Safe DateTime Parsing
  DateTime _safeParse(String? dateStr) {
    if (dateStr == null || dateStr.isEmpty) return DateTime.now();
    try {
      return DateTime.parse(dateStr);
    } catch (_) {
      return DateTime.now();
    }
  }

  // 🔄 Sync: Remote to Local
  Future<void> syncConversations() async {
    final response = await _apiClient.dio.get('/conversations');
    if (response.statusCode == 200) {
      final List<dynamic> jsonList = response.data['data'] ?? [];
      final List<db.ConversationsCompanion> companions = jsonList.map((e) {
        final conv = conv_model.Conversation.fromJson(e);
        return db.ConversationsCompanion.insert(
          id: conv.id,
          name: Value(conv.name),
          cumulativeTokens: Value(conv.cumulativeTokens),
          maxTokenUsage: Value(conv.maxTokenUsage),
          createdAt: _safeParse(conv.createdAt),
          updatedAt: _safeParse(conv.updatedAt),
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
          createdAt: _safeParse(m.createdAt),
          metadata: Value(m.metadata != null ? jsonEncode(m.metadata) : null),
          replyToId: Value(m.replyToId),
          repliedMessage: Value(m.repliedMessage != null ? jsonEncode(m.repliedMessage!.toJson()) : null),
          syncStatus: db.SyncStatus.synced,
        );
      }).toList();
      await _db.upsertMessages(companions);
    }
  }

  Future<void> syncReminders() async {
    final response = await _apiClient.dio.get('/reminders');
    if (response.statusCode == 200) {
      final List<dynamic> jsonList = response.data['data'] ?? [];
      final List<db.RemindersCompanion> companions = jsonList.map((e) {
        final r = rem_model.Reminder.fromJson(e);
        return db.RemindersCompanion.insert(
          id: r.id,
          content: r.content,
          taskType: Value(r.taskType),
          frequency: Value(r.frequency),
          status: r.status,
          dueAt: _safeParse(r.dueAt),
          createdAt: _safeParse(r.createdAt),
          userDisplayName: Value(r.userDisplayName),
          conversationTitle: Value(r.conversationTitle),
        );
      }).toList();
      await _db.upsertReminders(companions);
    }
  }

  Future<void> syncTransactions({int page = 1, String? query, String? category}) async {
    final Map<String, dynamic> params = {'page': page};
    if (query != null && query.isNotEmpty) params['query'] = query;
    if (category != null && category.isNotEmpty) params['category'] = category;

    final response = await _apiClient.dio.get('/money/history', queryParameters: params);
    if (response.statusCode == 200) {
      final List<dynamic> jsonList = response.data['data']['items'] ?? [];
      final List<db.TransactionsCompanion> companions = jsonList.map((e) {
        final tx = tx_model.Transaction.fromJson(e);
        return db.TransactionsCompanion.insert(
          id: tx.id,
          merchantName: Value(tx.merchantName),
          category: Value(tx.category),
          description: Value(tx.description),
          totalAmount: tx.totalAmount,
          createdAt: _safeParse(tx.createdAt),
          userDisplayName: Value(tx.userDisplayName),
          conversationTitle: Value(tx.conversationTitle),
          itemsJson: Value(tx.items != null ? jsonEncode(tx.items!.map((i) => i.toJson()).toList()) : null),
        );
      }).toList();
      await _db.upsertTransactions(companions);
    }
  }

  Future<void> syncHealthHistory({DateTime? start, DateTime? end}) async {
    final Map<String, dynamic> params = {};
    if (start != null) params['start_date'] = start.toIso8601String().split('T')[0];
    if (end != null) params['end_date'] = end.toIso8601String().split('T')[0];

    final response = await _apiClient.dio.get('/health/history', queryParameters: params);
    if (response.statusCode == 200) {
      final List<dynamic> jsonList = response.data['data'] ?? [];
      final List<db.HealthMetricsCompanion> companions = jsonList.map((e) {
        final m = health_model.HealthMetric.fromJson(e);
        return db.HealthMetricsCompanion.insert(
          id: m.id,
          userId: m.userId,
          logDate: _safeParse(m.logDate),
          steps: Value(m.metrics.steps),
          avgHeartRate: Value(m.metrics.avgHeartRate),
          sleepHours: Value(m.metrics.sleepHours),
          updatedAt: _safeParse(m.updatedAt),
        );
      }).toList();
      await _db.upsertHealthMetrics(companions);
    }
  }

  Future<void> syncPlugins() async {
    final response = await _apiClient.dio.get('/plugins');
    if (response.statusCode == 200) {
      final List<dynamic> jsonList = response.data['data'] ?? [];
      final List<db.PluginsCompanion> companions = jsonList.map((e) {
        final p = plugin_model.Plugin.fromJson(e);
        return db.PluginsCompanion.insert(
          id: p.id,
          name: p.name,
          slug: p.slug,
          description: Value(p.description),
          scriptCode: Value(p.scriptCode),
          version: Value(p.version),
          author: Value(p.author),
          intentsJson: Value(p.intents != null ? jsonEncode(p.intents) : null),
          createdAt: _safeParse(p.createdAt),
          updatedAt: _safeParse(p.updatedAt),
        );
      }).toList();
      await _db.upsertPlugins(companions);
    }
  }

  // 🌍 SRP Factory Access
  Future<List<srp_model.SrpProposal>> getProposals() async {
    final response = await _apiClient.dio.get('/srp/proposals');
    if (response.statusCode == 200) {
      final List<dynamic> list = response.data['data'] ?? [];
      return list.map((e) => srp_model.SrpProposal.fromJson(e)).toList();
    }
    return [];
  }

  Future<List<String>> getAvailablePlugins() async {
    final response = await _apiClient.dio.get('/srp/available');
    if (response.statusCode == 200) {
      return List<String>.from(response.data['data'] ?? []);
    }
    return [];
  }

  // 🧠 Reinforcement Access
  Future<reinforcement_model.ReinforcementState?> getReinforcement(String slug) async {
    try {
      final response = await _apiClient.dio.get('/srp/$slug');
      if (response.statusCode == 200) {
        return reinforcement_model.ReinforcementState.fromJson(response.data['data']);
      }
    } on DioException catch (e) {
      if (e.response?.statusCode == 404) {
        return reinforcement_model.ReinforcementState(
          slug: slug,
          enrichedDescription: "Original static definition active. No autonomous optimizations detected.",
          additionalRules: [],
          learnedPhrases: [],
        );
      }
    } catch (_) {}
    return null;
  }

  Future<String?> testSrp(String slug, String text) async {
    try {
      final response = await _apiClient.dio.post('/srp/test', data: {'slug': slug, 'text': text});
      if (response.statusCode == 200) {
        return response.data['data']['outcome'] as String?;
      }
    } catch (_) {}
    return null;
  }

  // 🛡️ Guardrail Access
  Future<List<guardrail_model.GuardrailPattern>> getGuardrailPatterns() async {
    final response = await _apiClient.dio.get('/v1/admin/guardrails/patterns');
    if (response.statusCode == 200) {
      final List<dynamic> list = response.data['data'] ?? [];
      return list.map((e) => guardrail_model.GuardrailPattern.fromJson(e)).toList();
    }
    return [];
  }

  Future<bool> createGuardrailPattern(String content) async {
    try {
      final response = await _apiClient.dio.post('/v1/admin/guardrails/patterns', data: {'content': content});
      return response.statusCode == 200 || response.statusCode == 201;
    } catch (_) {
      return false;
    }
  }

  Future<bool> deleteGuardrailPattern(String id) async {
    try {
      final response = await _apiClient.dio.delete('/v1/admin/guardrails/patterns/$id');
      return response.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // 🛠️ Skills Access
  Future<List<skill_model.Skill>> getSkills() async {
    final response = await _apiClient.dio.get('/skills');
    if (response.statusCode == 200) {
      final List<dynamic> list = response.data['data'] ?? [];
      return list.map((e) => skill_model.Skill.fromJson(e)).toList();
    }
    return [];
  }

  // 📊 Admin Monitor Access
  Future<List<admin_conv_model.AdminConversation>> getAdminConversations({String? cursor}) async {
    final Map<String, dynamic> params = {'limit': 50};
    if (cursor != null) params['cursor'] = cursor;
    final response = await _apiClient.dio.get('/v1/admin/conversations', queryParameters: params);
    if (response.statusCode == 200) {
      final data = response.data['data'];
      if (data != null && data['items'] != null) {
        final List<dynamic> list = data['items'];
        return list.map((e) => admin_conv_model.AdminConversation.fromJson(e)).toList();
      }
    }
    return [];
  }

  Future<bool> updateAdminConversation(String id, {int? maxTokenUsage, String? title, Map<String, dynamic>? thresholds}) async {
    final Map<String, dynamic> data = {};
    if (maxTokenUsage != null) data['max_token_usage'] = maxTokenUsage;
    if (title != null) data['title'] = title;
    if (thresholds != null) data['thresholds'] = thresholds;

    try {
      final response = await _apiClient.dio.patch('/v1/admin/conversations/$id', data: data);
      return response.statusCode == 200;
    } catch (_) {
      return false;
    }
  }

  // 👥 User Directory Access
  Future<List<user_model.UserProfile>> getUsers({String? query}) async {
    final Map<String, dynamic> params = {};
    if (query != null && query.isNotEmpty) params['query'] = query;
    final response = await _apiClient.dio.get('/v1/admin/users', queryParameters: params);
    if (response.statusCode == 200) {
      final data = response.data['data'];
      if (data != null && data['items'] != null) {
         final List<dynamic> list = data['items'];
         return list.map((e) => user_model.UserProfile.fromJson(e)).toList();
      }
      return [];
    }
    return [];
  }

  Future<user_detail_model.UserDetail?> getUserDetail(String id) async {
    final response = await _apiClient.dio.get('/v1/admin/users/$id');
    if (response.statusCode == 200) {
      final Map<String, dynamic> data = response.data['data'];
      return user_detail_model.UserDetail.fromJson(data);
    }
    return null;
  }

  // 💾 Storage Monitor Access
  Future<List<storage_model.StorageItem>> exploreStorage({String? prefix}) async {
    final Map<String, dynamic> params = {};
    if (prefix != null && prefix.isNotEmpty) params['prefix'] = prefix;
    final response = await _apiClient.dio.get('/v1/admin/storage/explore', queryParameters: params);
    if (response.statusCode == 200) {
      final List<dynamic> list = response.data['data'] ?? [];
      return list.map((e) => storage_model.StorageItem.fromJson(e)).toList();
    }
    return [];
  }

  Future<rem_model.Reminder?> getReminder(String id) async {
    final local = await _db.getReminderById(id);
    if (local != null) return rem_model.Reminder.fromDb(local);

    try {
      final response = await _apiClient.dio.get('/reminders/$id');
      if (response.statusCode == 200) {
        final r = rem_model.Reminder.fromJson(response.data['data']);
        await _db.upsertReminders([
          db.RemindersCompanion.insert(
            id: r.id,
            content: r.content,
            taskType: Value(r.taskType),
            frequency: Value(r.frequency),
            status: r.status,
            dueAt: _safeParse(r.dueAt),
            createdAt: _safeParse(r.createdAt),
            userDisplayName: Value(r.userDisplayName),
            conversationTitle: Value(r.conversationTitle),
          )
        ]);
        return r;
      }
    } catch (_) {}
    return null;
  }

  Future<tx_model.Transaction?> getTransaction(String id) async {
    final local = await _db.getTransactionById(id);
    if (local != null) return tx_model.Transaction.fromDb(local);

    try {
      final response = await _apiClient.dio.get('/money/history/$id');
      if (response.statusCode == 200) {
        final tx = tx_model.Transaction.fromJson(response.data['data']);
        await _db.upsertTransactions([
          db.TransactionsCompanion.insert(
            id: tx.id,
            merchantName: Value(tx.merchantName),
            category: Value(tx.category),
            description: Value(tx.description),
            totalAmount: tx.totalAmount,
            createdAt: _safeParse(tx.createdAt),
            userDisplayName: Value(tx.userDisplayName),
            conversationTitle: Value(tx.conversationTitle),
            itemsJson: Value(tx.items != null ? jsonEncode(tx.items!.map((i) => i.toJson()).toList()) : null),
          )
        ]);
        return tx;
      }
    } catch (_) {}
    return null;
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
