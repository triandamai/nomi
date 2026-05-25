import 'package:json_annotation/json_annotation.dart';

part 'conversation.g.dart';

@JsonSerializable()
class Conversation {
  final String id;
  final String? name;
  @JsonKey(name: 'cumulative_tokens')
  final int? cumulativeTokens;
  @JsonKey(name: 'max_token_usage')
  final int? maxTokenUsage;
  @JsonKey(name: 'gateway_thresholds')
  final Map<String, dynamic>? gatewayThresholds;
  @JsonKey(name: 'created_at')
  final String createdAt;
  @JsonKey(name: 'updated_at')
  final String updatedAt;

  Conversation({
    required this.id,
    this.name,
    this.cumulativeTokens,
    this.maxTokenUsage,
    this.gatewayThresholds,
    required this.createdAt,
    required this.updatedAt,
  });

  factory Conversation.fromJson(Map<String, dynamic> json) => _$ConversationFromJson(json);
  Map<String, dynamic> toJson() => _$ConversationToJson(this);

  String get displayName => name ?? 'Private Sandbox';
}

@JsonSerializable()
class Profile {
  final String id;
  @JsonKey(name: 'external_id')
  final String? externalId;
  @JsonKey(name: 'display_name')
  final String? displayName;
  @JsonKey(name: 'avatar_url')
  final String? avatarUrl;
  final String? role;

  Profile({
    required this.id,
    this.externalId,
    this.displayName,
    this.avatarUrl,
    this.role,
  });

  factory Profile.fromJson(Map<String, dynamic> json) => _$ProfileFromJson(json);
  Map<String, dynamic> toJson() => _$ProfileToJson(this);
}
