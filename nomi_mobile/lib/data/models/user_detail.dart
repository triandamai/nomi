import 'package:json_annotation/json_annotation.dart';

part 'user_detail.g.dart';

@JsonSerializable()
class UserDetail {
  @JsonKey(fromJson: _parseStringRequired)
  final String id;
  final String? name;
  @JsonKey(name: 'display_name')
  final String? displayName;
  final String? email;
  final String? role;
  @JsonKey(name: 'is_verified')
  final bool? isVerified;
  @JsonKey(name: 'created_at')
  final String? createdAt;
  
  @JsonKey(defaultValue: [])
  final List<UserChannel> channels;
  @JsonKey(defaultValue: [])
  final List<UserConversation> conversations;

  UserDetail({
    required this.id,
    this.name,
    this.displayName,
    this.email,
    this.role,
    this.isVerified,
    this.createdAt,
    required this.channels,
    required this.conversations,
  });

  static String _parseStringRequired(dynamic value) {
    if (value == null) return '';
    return value.toString();
  }

  factory UserDetail.fromJson(Map<String, dynamic> json) => _$UserDetailFromJson(json);
  Map<String, dynamic> toJson() => _$UserDetailToJson(this);
}

@JsonSerializable()
class UserChannel {
  @JsonKey(fromJson: _parseStringRequired)
  final String id;
  @JsonKey(name: 'channel_type', fromJson: _parseStringRequired)
  final String channelType;
  @JsonKey(name: 'conversation_title')
  final String? conversationTitle;

  UserChannel({required this.id, required this.channelType, this.conversationTitle});

  static String _parseStringRequired(dynamic value) {
    if (value == null) return '';
    return value.toString();
  }

  factory UserChannel.fromJson(Map<String, dynamic> json) => _$UserChannelFromJson(json);
  Map<String, dynamic> toJson() => _$UserChannelToJson(this);
}

@JsonSerializable()
class UserConversation {
  @JsonKey(name: 'conversation_id', fromJson: _parseStringRequired)
  final String conversationId;
  final String? title;
  @JsonKey(name: 'joined_at', fromJson: _parseStringRequired)
  final String joinedAt;

  UserConversation({required this.conversationId, this.title, required this.joinedAt});

  static String _parseStringRequired(dynamic value) {
    if (value == null) return '';
    return value.toString();
  }

  factory UserConversation.fromJson(Map<String, dynamic> json) => _$UserConversationFromJson(json);
  Map<String, dynamic> toJson() => _$UserConversationToJson(this);
}
