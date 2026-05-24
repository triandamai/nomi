import 'dart:convert';
import 'package:json_annotation/json_annotation.dart';

part 'message.g.dart';

@JsonSerializable()
class Message {
  final String id;
  @JsonKey(name: 'conversation_id')
  final String conversationId;
  final String role;
  final String content;
  @JsonKey(name: 'display_name')
  final String? displayName;
  final String? thought;
  @JsonKey(name: 'image_url')
  final String? imageUrl;
  final String? video_url;
  final String? audio_url;
  final String? document_url;
  final String? sticker_url;
  @JsonKey(name: 'user_id')
  final String? userId;
  @JsonKey(name: 'total_tokens')
  final int? totalTokens;
  @JsonKey(name: 'created_at')
  final String createdAt;
  final Map<String, dynamic>? metadata;
  @JsonKey(name: 'reply_to_id')
  final String? replyToId;
  @JsonKey(name: 'replied_message')
  final RepliedMessage? repliedMessage;

  Message({
    required this.id,
    required this.conversationId,
    required this.role,
    required this.content,
    this.displayName,
    this.thought,
    this.imageUrl,
    this.video_url,
    this.audio_url,
    this.document_url,
    this.sticker_url,
    this.userId,
    this.totalTokens,
    required this.createdAt,
    this.metadata,
    this.replyToId,
    this.repliedMessage,
  });

  factory Message.fromJson(Map<String, dynamic> json) => _$MessageFromJson(json);
  Map<String, dynamic> toJson() => _$MessageToJson(this);

  // 🏛️ Bridge: From Drift to UI Model
  factory Message.fromDb(dynamic dbMsg) {
    // Note: using dynamic because Drift types are in database.g.dart
    final m = dbMsg; 
    
    Map<String, dynamic>? meta;
    if (m.metadata != null) {
      try {
        meta = Map<String, dynamic>.from(jsonDecode(m.metadata!));
      } catch (_) {}
    }

    RepliedMessage? replied;
    if (m.repliedMessage != null) {
      try {
        replied = RepliedMessage.fromJson(jsonDecode(m.repliedMessage!));
      } catch (_) {}
    }

    return Message(
      id: m.id,
      conversationId: m.conversationId,
      role: m.role,
      content: m.content,
      displayName: m.displayName,
      thought: m.thought,
      imageUrl: m.imageUrl,
      video_url: m.videoUrl,
      audio_url: m.audioUrl,
      document_url: m.documentUrl,
      sticker_url: m.stickerUrl,
      userId: m.userId,
      totalTokens: m.totalTokens,
      createdAt: m.createdAt.toIso8601String(),
      metadata: meta,
      replyToId: m.replyToId,
      repliedMessage: replied,
    );
  }

  bool get isUser => role == 'user';
  bool get isAssistant => role == 'assistant';
  bool get isSystem => role == 'system';
}

@JsonSerializable()
class RepliedMessage {
  final String id;
  final String role;
  final String content;
  @JsonKey(name: 'display_name')
  final String? displayName;

  RepliedMessage({
    required this.id,
    required this.role,
    required this.content,
    this.displayName,
  });

  factory RepliedMessage.fromJson(Map<String, dynamic> json) => _$RepliedMessageFromJson(json);
  Map<String, dynamic> toJson() => _$RepliedMessageToJson(this);
}
