import 'package:json_annotation/json_annotation.dart';

part 'skill.g.dart';

@JsonSerializable()
class Skill {
  final String name;
  final String description;
  final List<String> intents;
  @JsonKey(name: 'skill_type')
  final String skillType; // 'System' | 'Dynamic'
  @JsonKey(name: 'script_code')
  final String? scriptCode;
  @JsonKey(name: 'schema_json')
  final Map<String, dynamic>? schemaJson;
  @JsonKey(name: 'creator_name')
  final String? creatorName;

  Skill({
    required this.name,
    required this.description,
    required this.intents,
    required this.skillType,
    this.scriptCode,
    this.schemaJson,
    this.creatorName,
  });

  factory Skill.fromJson(Map<String, dynamic> json) => _$SkillFromJson(json);
  Map<String, dynamic> toJson() => _$SkillToJson(this);
}
