import 'package:json_annotation/json_annotation.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;

part 'plugin.g.dart';

@JsonSerializable()
class Plugin {
  @JsonKey(fromJson: _parseStringRequired)
  final String id;
  @JsonKey(fromJson: _parseStringRequired)
  final String name;
  @JsonKey(fromJson: _parseStringRequired)
  final String slug;
  final String? description;
  @JsonKey(name: 'script_code')
  final String? scriptCode;
  @JsonKey(fromJson: _parseString)
  final String? version;
  @JsonKey(name: 'display_name')
  final String? author;
  final List<String>? intents;
  @JsonKey(name: 'created_at', fromJson: _parseStringRequired)
  final String createdAt;
  @JsonKey(name: 'updated_at', fromJson: _parseStringRequired)
  final String updatedAt;

  Plugin({
    required this.id,
    required this.name,
    required this.slug,
    this.description,
    this.scriptCode,
    this.version,
    this.author,
    this.intents,
    required this.createdAt,
    required this.updatedAt,
  });

  static String _parseStringRequired(dynamic value) {
    if (value == null) return '';
    return value.toString();
  }

  static String? _parseString(dynamic value) {
    if (value == null) return null;
    return value.toString();
  }

  factory Plugin.fromJson(Map<String, dynamic> json) => _$PluginFromJson(json);
  Map<String, dynamic> toJson() => _$PluginToJson(this);

  // 🏛️ Bridge: From Drift to UI Model
  factory Plugin.fromDb(db.Plugin m) {
    return Plugin(
      id: m.id,
      name: m.name,
      slug: m.slug,
      description: m.description,
      scriptCode: m.scriptCode,
      version: m.version,
      author: m.author,
      createdAt: m.createdAt.toIso8601String(),
      updatedAt: m.updatedAt.toIso8601String(),
    );
  }
}
