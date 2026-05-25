// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'admin_conversation.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

AdminConversation _$AdminConversationFromJson(Map<String, dynamic> json) =>
    AdminConversation(
      id: json['id'] as String,
      title: json['title'] as String?,
      cumulativeTokens: (json['cumulative_tokens'] as num?)?.toInt(),
      maxTokenUsage: (json['max_token_usage'] as num?)?.toInt(),
      gatewayThresholds: json['gateway_thresholds'] as Map<String, dynamic>?,
      createdAt: json['created_at'] as String,
    );

Map<String, dynamic> _$AdminConversationToJson(AdminConversation instance) =>
    <String, dynamic>{
      'id': instance.id,
      'title': instance.title,
      'cumulative_tokens': instance.cumulativeTokens,
      'max_token_usage': instance.maxTokenUsage,
      'gateway_thresholds': instance.gatewayThresholds,
      'created_at': instance.createdAt,
    };
