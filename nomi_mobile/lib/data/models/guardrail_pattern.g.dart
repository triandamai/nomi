// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'guardrail_pattern.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

GuardrailPattern _$GuardrailPatternFromJson(Map<String, dynamic> json) =>
    GuardrailPattern(
      id: json['id'] as String,
      content: json['content'] as String,
      score: (json['score'] as num?)?.toDouble(),
      createdAt: json['created_at'] as String,
    );

Map<String, dynamic> _$GuardrailPatternToJson(GuardrailPattern instance) =>
    <String, dynamic>{
      'id': instance.id,
      'content': instance.content,
      'score': instance.score,
      'created_at': instance.createdAt,
    };
