// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'message.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Message _$MessageFromJson(Map<String, dynamic> json) => Message(
  id: json['id'] as String,
  conversationId: json['conversation_id'] as String,
  role: json['role'] as String,
  content: json['content'] as String,
  displayName: json['display_name'] as String?,
  thought: json['thought'] as String?,
  imageUrl: json['image_url'] as String?,
  video_url: json['video_url'] as String?,
  audio_url: json['audio_url'] as String?,
  document_url: json['document_url'] as String?,
  sticker_url: json['sticker_url'] as String?,
  userId: json['user_id'] as String?,
  totalTokens: (json['total_tokens'] as num?)?.toInt(),
  createdAt: json['created_at'] as String,
  metadata: json['metadata'] as Map<String, dynamic>?,
  replyToId: json['reply_to_id'] as String?,
  repliedMessage:
      json['replied_message'] == null
          ? null
          : RepliedMessage.fromJson(
            json['replied_message'] as Map<String, dynamic>,
          ),
);

Map<String, dynamic> _$MessageToJson(Message instance) => <String, dynamic>{
  'id': instance.id,
  'conversation_id': instance.conversationId,
  'role': instance.role,
  'content': instance.content,
  'display_name': instance.displayName,
  'thought': instance.thought,
  'image_url': instance.imageUrl,
  'video_url': instance.video_url,
  'audio_url': instance.audio_url,
  'document_url': instance.document_url,
  'sticker_url': instance.sticker_url,
  'user_id': instance.userId,
  'total_tokens': instance.totalTokens,
  'created_at': instance.createdAt,
  'metadata': instance.metadata,
  'reply_to_id': instance.replyToId,
  'replied_message': instance.repliedMessage,
};

RepliedMessage _$RepliedMessageFromJson(Map<String, dynamic> json) =>
    RepliedMessage(
      id: json['id'] as String,
      role: json['role'] as String,
      content: json['content'] as String,
      displayName: json['display_name'] as String?,
    );

Map<String, dynamic> _$RepliedMessageToJson(RepliedMessage instance) =>
    <String, dynamic>{
      'id': instance.id,
      'role': instance.role,
      'content': instance.content,
      'display_name': instance.displayName,
    };
