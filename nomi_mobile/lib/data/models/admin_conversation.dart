import 'package:json_annotation/json_annotation.dart';

part 'admin_conversation.g.dart';

@JsonSerializable()
class AdminConversation {
  final String id;
  final String? title;
  @JsonKey(name: 'cumulative_tokens')
  final int? cumulativeTokens;
  @JsonKey(name: 'max_token_usage')
  final int? maxTokenUsage;
  @JsonKey(name: 'gateway_thresholds')
  final Map<String, dynamic>? gatewayThresholds;
  @JsonKey(name: 'created_at')
  final String createdAt;

  AdminConversation({
    required this.id,
    this.title,
    this.cumulativeTokens,
    this.maxTokenUsage,
    this.gatewayThresholds,
    required this.createdAt,
  });

  factory AdminConversation.fromJson(Map<String, dynamic> json) => _$AdminConversationFromJson(json);
  Map<String, dynamic> toJson() => _$AdminConversationToJson(this);
}
