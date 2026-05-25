// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'plugin.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Plugin _$PluginFromJson(Map<String, dynamic> json) => Plugin(
  id: Plugin._parseStringRequired(json['id']),
  name: Plugin._parseStringRequired(json['name']),
  slug: Plugin._parseStringRequired(json['slug']),
  description: json['description'] as String?,
  scriptCode: json['script_code'] as String?,
  version: Plugin._parseString(json['version']),
  author: json['display_name'] as String?,
  intents:
      (json['intents'] as List<dynamic>?)?.map((e) => e as String).toList(),
  createdAt: Plugin._parseStringRequired(json['created_at']),
  updatedAt: Plugin._parseStringRequired(json['updated_at']),
);

Map<String, dynamic> _$PluginToJson(Plugin instance) => <String, dynamic>{
  'id': instance.id,
  'name': instance.name,
  'slug': instance.slug,
  'description': instance.description,
  'script_code': instance.scriptCode,
  'version': instance.version,
  'display_name': instance.author,
  'intents': instance.intents,
  'created_at': instance.createdAt,
  'updated_at': instance.updatedAt,
};
