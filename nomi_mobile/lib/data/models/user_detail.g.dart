// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'user_detail.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

UserDetail _$UserDetailFromJson(Map<String, dynamic> json) => UserDetail(
  id: UserDetail._parseStringRequired(json['id']),
  name: json['name'] as String?,
  displayName: json['display_name'] as String?,
  email: json['email'] as String?,
  role: json['role'] as String?,
  isVerified: json['is_verified'] as bool?,
  createdAt: json['created_at'] as String?,
  channels:
      (json['channels'] as List<dynamic>?)
          ?.map((e) => UserChannel.fromJson(e as Map<String, dynamic>))
          .toList() ??
      [],
  conversations:
      (json['conversations'] as List<dynamic>?)
          ?.map((e) => UserConversation.fromJson(e as Map<String, dynamic>))
          .toList() ??
      [],
);

Map<String, dynamic> _$UserDetailToJson(UserDetail instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'display_name': instance.displayName,
      'email': instance.email,
      'role': instance.role,
      'is_verified': instance.isVerified,
      'created_at': instance.createdAt,
      'channels': instance.channels,
      'conversations': instance.conversations,
    };

UserChannel _$UserChannelFromJson(Map<String, dynamic> json) => UserChannel(
  id: UserChannel._parseStringRequired(json['id']),
  channelType: UserChannel._parseStringRequired(json['channel_type']),
  conversationTitle: json['conversation_title'] as String?,
);

Map<String, dynamic> _$UserChannelToJson(UserChannel instance) =>
    <String, dynamic>{
      'id': instance.id,
      'channel_type': instance.channelType,
      'conversation_title': instance.conversationTitle,
    };

UserConversation _$UserConversationFromJson(Map<String, dynamic> json) =>
    UserConversation(
      conversationId: UserConversation._parseStringRequired(
        json['conversation_id'],
      ),
      title: json['title'] as String?,
      joinedAt: UserConversation._parseStringRequired(json['joined_at']),
    );

Map<String, dynamic> _$UserConversationToJson(UserConversation instance) =>
    <String, dynamic>{
      'conversation_id': instance.conversationId,
      'title': instance.title,
      'joined_at': instance.joinedAt,
    };
