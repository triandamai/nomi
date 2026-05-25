// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'skill.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Skill _$SkillFromJson(Map<String, dynamic> json) => Skill(
  name: json['name'] as String,
  description: json['description'] as String,
  intents: (json['intents'] as List<dynamic>).map((e) => e as String).toList(),
  skillType: json['skill_type'] as String,
  scriptCode: json['script_code'] as String?,
  schemaJson: json['schema_json'] as Map<String, dynamic>?,
  creatorName: json['creator_name'] as String?,
);

Map<String, dynamic> _$SkillToJson(Skill instance) => <String, dynamic>{
  'name': instance.name,
  'description': instance.description,
  'intents': instance.intents,
  'skill_type': instance.skillType,
  'script_code': instance.scriptCode,
  'schema_json': instance.schemaJson,
  'creator_name': instance.creatorName,
};
