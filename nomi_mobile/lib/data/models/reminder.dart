import 'package:json_annotation/json_annotation.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;

part 'reminder.g.dart';

@JsonSerializable()
class Reminder {
  final String id;
  final String content;
  @JsonKey(name: 'task_type')
  final String? taskType;
  final String? frequency;
  final String status;
  @JsonKey(name: 'due_at')
  final String dueAt;
  @JsonKey(name: 'created_at')
  final String createdAt;
  @JsonKey(name: 'user_display_name')
  final String? userDisplayName;
  @JsonKey(name: 'conversation_title')
  final String? conversationTitle;

  Reminder({
    required this.id,
    required this.content,
    this.taskType,
    this.frequency,
    required this.status,
    required this.dueAt,
    required this.createdAt,
    this.userDisplayName,
    this.conversationTitle,
  });

  factory Reminder.fromJson(Map<String, dynamic> json) => _$ReminderFromJson(json);
  Map<String, dynamic> toJson() => _$ReminderToJson(this);

  // 🏛️ Bridge: From Drift to UI Model
  factory Reminder.fromDb(db.Reminder m) {
    return Reminder(
      id: m.id,
      content: m.content,
      taskType: m.taskType,
      frequency: m.frequency,
      status: m.status,
      dueAt: m.dueAt.toIso8601String(),
      createdAt: m.createdAt.toIso8601String(),
      userDisplayName: m.userDisplayName,
      conversationTitle: m.conversationTitle,
    );
  }
}
