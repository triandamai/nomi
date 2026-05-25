import 'package:json_annotation/json_annotation.dart';

part 'guardrail_pattern.g.dart';

@JsonSerializable()
class GuardrailPattern {
  final String id;
  final String content;
  final double? score;
  @JsonKey(name: 'created_at')
  final String createdAt;

  GuardrailPattern({
    required this.id,
    required this.content,
    this.score,
    required this.createdAt,
  });

  factory GuardrailPattern.fromJson(Map<String, dynamic> json) => _$GuardrailPatternFromJson(json);
  Map<String, dynamic> toJson() => _$GuardrailPatternToJson(this);
}
