// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'health_metric.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

HealthMetric _$HealthMetricFromJson(Map<String, dynamic> json) => HealthMetric(
  id: json['id'] as String,
  userId: json['user_id'] as String,
  logDate: json['log_date'] as String,
  metrics: HealthMetricsData.fromJson(json['metrics'] as Map<String, dynamic>),
  updatedAt: json['updated_at'] as String,
);

Map<String, dynamic> _$HealthMetricToJson(HealthMetric instance) =>
    <String, dynamic>{
      'id': instance.id,
      'user_id': instance.userId,
      'log_date': instance.logDate,
      'metrics': instance.metrics,
      'updated_at': instance.updatedAt,
    };

HealthMetricsData _$HealthMetricsDataFromJson(Map<String, dynamic> json) =>
    HealthMetricsData(
      steps: (json['steps'] as num?)?.toInt() ?? 0,
      avgHeartRate: (json['avg_heart_rate'] as num?)?.toInt(),
      sleepHours: (json['sleep_hours'] as num?)?.toDouble(),
    );

Map<String, dynamic> _$HealthMetricsDataToJson(HealthMetricsData instance) =>
    <String, dynamic>{
      'steps': instance.steps,
      'avg_heart_rate': instance.avgHeartRate,
      'sleep_hours': instance.sleepHours,
    };
