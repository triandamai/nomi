import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:mqtt_client/mqtt_client.dart';
import 'package:nomi_mobile/data/models/message.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/core/api/mqtt_service.dart';
import 'package:nomi_mobile/providers/database_provider.dart';
import 'package:drift/drift.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;

class ChatState {
  final Map<String, String> thoughts;
  final Map<String, String?> activeTools;
  final Map<String, bool> isTyping;
  final String? activeConversationId;
  final bool isLoading;
  final String? error;
  final Message? replyingTo;
  final bool isSidebarExpanded;

  ChatState({
    this.thoughts = const {},
    this.activeTools = const {},
    this.isTyping = const {},
    this.activeConversationId,
    this.isLoading = false,
    this.error,
    this.replyingTo,
    this.isSidebarExpanded = false,
  });

  ChatState copyWith({
    Map<String, String>? thoughts,
    Map<String, String?>? activeTools,
    Map<String, bool>? isTyping,
    String? activeConversationId,
    bool? isLoading,
    String? error,
    Message? replyingTo,
    bool clearReplyingTo = false,
    bool? isSidebarExpanded,
  }) {
    return ChatState(
      thoughts: thoughts ?? this.thoughts,
      activeTools: activeTools ?? this.activeTools,
      isTyping: isTyping ?? this.isTyping,
      activeConversationId: activeConversationId ?? this.activeConversationId,
      isLoading: isLoading ?? this.isLoading,
      error: error,
      replyingTo: clearReplyingTo ? null : (replyingTo ?? this.replyingTo),
      isSidebarExpanded: isSidebarExpanded ?? this.isSidebarExpanded,
    );
  }
}

class ChatNotifier extends Notifier<ChatState> {
  @override
  ChatState build() {
    // 🌐 Listening to the global MQTT stream directly (UI Isolate)
    final mqtt = ref.read(mqttServiceProvider);

    mqtt.updates.listen((List<MqttReceivedMessage<MqttMessage>> c) {
      if (c.isEmpty) return;

      final MqttPublishMessage rec = c[0].payload as MqttPublishMessage;
      final String payload = MqttPublishPayload.bytesToStringAsString(
        rec.payload.message,
      );
      final Map<String, dynamic> data = jsonDecode(payload);
      final String topic = c[0].topic;

      if (topic.contains('/message')) {
        _handleIncomingMessage(data);
      } else if (topic.contains('/thought')) {
        _handleThought(data);
      } else if (topic.contains('/tool_start')) {
        _handleToolStart(data);
      } else if (topic.contains('/tool_end')) {
        _handleToolEnd(data);
      } else if (topic.contains('/presence')) {
        _handlePresence(data);
      }
    });

    // 💾 Load persistent session state
    _loadPersistedState();

    return ChatState();
  }

  Future<void> _loadPersistedState() async {
    final prefs = await SharedPreferences.getInstance();
    final String? lastId = prefs.getString('active_conversation_id');
    if (lastId != null) {
      state = state.copyWith(activeConversationId: lastId);
      // Re-sync messages for the persisted session
      ref.read(chatRepositoryProvider).syncMessages(lastId);
    }
  }

  Future<void> _handleIncomingMessage(Map<String, dynamic> data) async {
    final msg = Message.fromJson(data);
    final convId = msg.conversationId;
    final db_client = ref.read(databaseProvider);

    // 💡 Logic to prevent duplication: 
    // If this is a 'user' message, check if we have a matching 'pending' message locally.
    // We match by content and role because the temp ID won't match the server UUID.
    if (msg.role == 'user') {
      final pending = await db_client.getPendingMessages();
      final match = pending.where((m) => m.content == msg.content && m.conversationId == convId).toList();
      
      if (match.isNotEmpty) {
        for (var m in match) {
          await db_client.customStatement('DELETE FROM messages WHERE id = ?', [m.id]);
        }
      }
    }

    // 💾 Durable Persistence: MQTT ➡️ Local DB
    final companion = db.MessagesCompanion.insert(
      id: msg.id,
      conversationId: msg.conversationId,
      role: msg.role,
      content: msg.content,
      displayName: Value(msg.displayName),
      thought: Value(msg.thought),
      imageUrl: Value(msg.imageUrl),
      videoUrl: Value(msg.video_url),
      audioUrl: Value(msg.audio_url),
      documentUrl: Value(msg.document_url),
      stickerUrl: Value(msg.sticker_url),
      userId: Value(msg.userId),
      totalTokens: Value(msg.totalTokens),
      createdAt: DateTime.parse(msg.createdAt),
      metadata: Value(msg.metadata != null ? jsonEncode(msg.metadata) : null),
      replyToId: Value(msg.replyToId),
      repliedMessage: Value(msg.repliedMessage != null ? jsonEncode(msg.repliedMessage!.toJson()) : null),
      syncStatus: db.SyncStatus.synced,
    );

    await db_client.upsertMessages([companion]);

    // Clear temporary thought/tool/typing state
    final updatedThoughts = Map<String, String>.from(state.thoughts);
    updatedThoughts[convId] = "";

    final updatedTools = Map<String, String?>.from(state.activeTools);
    updatedTools[convId] = null;

    final updatedTyping = Map<String, bool>.from(state.isTyping);
    updatedTyping[convId] = false;

    state = state.copyWith(
      thoughts: updatedThoughts,
      activeTools: updatedTools,
      isTyping: updatedTyping,
    );
  }

  void _handleThought(Map<String, dynamic> data) {
    final convId = data['conversation_id'];
    final thought = data['thought'] ?? data['text'] ?? "";

    final updatedThoughts = Map<String, String>.from(state.thoughts);
    updatedThoughts[convId] = (updatedThoughts[convId] ?? "") + thought;

    state = state.copyWith(thoughts: updatedThoughts);
  }

  void _handleToolStart(Map<String, dynamic> data) {
    final convId = data['conversation_id'];
    final toolName = data['tool_name'] ?? data['name'] ?? "tool";

    final updatedTools = Map<String, String?>.from(state.activeTools);
    updatedTools[convId] = toolName;

    state = state.copyWith(activeTools: updatedTools);
  }

  void _handleToolEnd(Map<String, dynamic> data) {
    final convId = data['conversation_id'];

    final updatedTools = Map<String, String?>.from(state.activeTools);
    updatedTools[convId] = null;

    state = state.copyWith(activeTools: updatedTools);
  }

  void _handlePresence(Map<String, dynamic> data) {
    final convId = data['conversation_id'];
    final isTyping = data['is_typing'] ?? false;

    final updatedTyping = Map<String, bool>.from(state.isTyping);
    updatedTyping[convId] = isTyping;

    state = state.copyWith(isTyping: updatedTyping);
  }

  Future<void> fetchConversations() async {
    state = state.copyWith(isLoading: true);
    try {
      await ref.read(chatRepositoryProvider).syncConversations();
      state = state.copyWith(isLoading: false);
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
    }
  }

  Future<void> setActiveConversation(String id) async {
    state = state.copyWith(activeConversationId: id);
    
    // 💾 Persist technical session context
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString('active_conversation_id', id);

    // Trigger background sync
    ref.read(chatRepositoryProvider).syncMessages(id);
  }

  Future<void> fetchMessages(String conversationId, {bool loadMore = false}) async {
    // In local-first, the UI watches the DB. This just triggers a re-sync.
    await ref.read(chatRepositoryProvider).syncMessages(conversationId);
  }

  void setReplyingTo(Message? message) {
    state = state.copyWith(
      replyingTo: message,
      clearReplyingTo: message == null,
    );
  }

  void toggleSidebar() {
    state = state.copyWith(isSidebarExpanded: !state.isSidebarExpanded);
  }

  Future<void> sendMessage(String content, {Map<String, String>? media}) async {
    final convId = state.activeConversationId;
    if (convId == null) return;

    final replyMessage = state.replyingTo;
    final replyToId = replyMessage?.id;
    
    RepliedMessage? repliedMsg;
    if (replyMessage != null) {
      repliedMsg = RepliedMessage(
        id: replyMessage.id,
        role: replyMessage.role,
        content: replyMessage.content,
        displayName: replyMessage.displayName,
      );
    }

    state = state.copyWith(clearReplyingTo: true);

    try {
      await ref.read(chatRepositoryProvider).sendChatMessage(
            convId,
            content,
            replyToId: replyToId,
            repliedMessage: repliedMsg,
            media: media,
          );
    } catch (e) {
      state = state.copyWith(error: e.toString());
    }
  }
}

final chatProvider = NotifierProvider<ChatNotifier, ChatState>(() {
  return ChatNotifier();
});

final conversationsStreamProvider = StreamProvider<List<db.Conversation>>((ref) {
  return ref.watch(chatRepositoryProvider).watchConversations();
});

final messagesStreamProvider = StreamProvider.family<List<db.Message>, String>((ref, conversationId) {
  return ref.watch(chatRepositoryProvider).watchMessages(conversationId);
});
