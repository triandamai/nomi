// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'conversation.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Conversation _$ConversationFromJson(Map<String, dynamic> json) => Conversation(
  id: json['id'] as String,
  name: json['name'] as String?,
  cumulativeTokens: (json['cumulative_tokens'] as num?)?.toInt(),
  maxTokenUsage: (json['max_token_usage'] as num?)?.toInt(),
  createdAt: json['created_at'] as String,
  updatedAt: json['updated_at'] as String,
);

Map<String, dynamic> _$ConversationToJson(Conversation instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'cumulative_tokens': instance.cumulativeTokens,
      'max_token_usage': instance.maxTokenUsage,
      'created_at': instance.createdAt,
      'updated_at': instance.updatedAt,
    };

Profile _$ProfileFromJson(Map<String, dynamic> json) => Profile(
  id: json['id'] as String,
  externalId: json['external_id'] as String?,
  displayName: json['display_name'] as String?,
  avatarUrl: json['avatar_url'] as String?,
  role: json['role'] as String?,
);

Map<String, dynamic> _$ProfileToJson(Profile instance) => <String, dynamic>{
  'id': instance.id,
  'external_id': instance.externalId,
  'display_name': instance.displayName,
  'avatar_url': instance.avatarUrl,
  'role': instance.role,
};
