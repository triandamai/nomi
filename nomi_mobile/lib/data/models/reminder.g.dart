// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'reminder.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Reminder _$ReminderFromJson(Map<String, dynamic> json) => Reminder(
  id: json['id'] as String,
  content: json['content'] as String,
  taskType: json['task_type'] as String?,
  frequency: json['frequency'] as String?,
  status: json['status'] as String,
  dueAt: json['due_at'] as String,
  createdAt: json['created_at'] as String,
  userDisplayName: json['user_display_name'] as String?,
  conversationTitle: json['conversation_title'] as String?,
);

Map<String, dynamic> _$ReminderToJson(Reminder instance) => <String, dynamic>{
  'id': instance.id,
  'content': instance.content,
  'task_type': instance.taskType,
  'frequency': instance.frequency,
  'status': instance.status,
  'due_at': instance.dueAt,
  'created_at': instance.createdAt,
  'user_display_name': instance.userDisplayName,
  'conversation_title': instance.conversationTitle,
};
