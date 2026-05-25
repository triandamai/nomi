import 'package:json_annotation/json_annotation.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;

part 'health_metric.g.dart';

@JsonSerializable()
class HealthMetric {
  final String id;
  @JsonKey(name: 'user_id')
  final String userId;
  @JsonKey(name: 'log_date')
  final String logDate;
  final HealthMetricsData metrics;
  @JsonKey(name: 'updated_at')
  final String updatedAt;

  HealthMetric({
    required this.id,
    required this.userId,
    required this.logDate,
    required this.metrics,
    required this.updatedAt,
  });

  factory HealthMetric.fromJson(Map<String, dynamic> json) => _$HealthMetricFromJson(json);
  Map<String, dynamic> toJson() => _$HealthMetricToJson(this);

  // 🏛️ Bridge: From Drift to UI Model
  factory HealthMetric.fromDb(db.HealthMetric m) {
    return HealthMetric(
      id: m.id,
      userId: m.userId,
      logDate: m.logDate.toIso8601String(),
      metrics: HealthMetricsData(
        steps: m.steps,
        avgHeartRate: m.avgHeartRate,
        sleepHours: m.sleepHours,
      ),
      updatedAt: m.updatedAt.toIso8601String(),
    );
  }
}

@JsonSerializable()
class HealthMetricsData {
  final int steps;
  @JsonKey(name: 'avg_heart_rate')
  final int? avgHeartRate;
  @JsonKey(name: 'sleep_hours')
  final double? sleepHours;

  HealthMetricsData({
    this.steps = 0,
    this.avgHeartRate,
    this.sleepHours,
  });

  factory HealthMetricsData.fromJson(Map<String, dynamic> json) => _$HealthMetricsDataFromJson(json);
  Map<String, dynamic> toJson() => _$HealthMetricsDataToJson(this);
}
