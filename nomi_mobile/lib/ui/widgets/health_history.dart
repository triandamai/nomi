import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/data/models/health_metric.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;
import 'package:intl/intl.dart';
import 'dart:ui';
import 'dart:math' as math;

class HealthHistorySheet extends ConsumerStatefulWidget {
  const HealthHistorySheet({super.key});

  @override
  ConsumerState<HealthHistorySheet> createState() => _HealthHistorySheetState();
}

class _HealthHistorySheetState extends ConsumerState<HealthHistorySheet> {
  DateTime _startDate = DateTime.now().subtract(const Duration(days: 7));
  DateTime _endDate = DateTime.now();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _syncData();
    });
  }

  void _syncData() {
    ref.read(chatRepositoryProvider).syncHealthHistory(start: _startDate, end: _endDate);
  }

  @override
  Widget build(BuildContext context) {
    final historyStream = ref.watch(healthHistoryStreamProvider((_startDate, _endDate)));
    final size = MediaQuery.of(context).size;

    return Container(
      width: double.infinity,
      constraints: BoxConstraints(maxHeight: size.height * 0.9),
      decoration: BoxDecoration(
        color: const Color(AppConfig.deepSlate).withValues(alpha: 0.95),
        border: Border(top: BorderSide(color: Colors.white.withValues(alpha: 0.1))),
      ),
      child: Column(
        children: [
          // Header with Liquid Glass Feel
          ClipRRect(
            child: BackdropFilter(
              filter: ImageFilter.blur(sigmaX: 10, sigmaY: 10),
              child: Container(
                padding: const EdgeInsets.all(24),
                decoration: BoxDecoration(
                  color: Colors.white.withValues(alpha: 0.02),
                  border: Border(bottom: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
                ),
                child: Column(
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        const Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'BIOMETRIC DATA',
                              style: TextStyle(
                                color: Color(AppConfig.emerald),
                                fontSize: 10,
                                fontWeight: FontWeight.w900,
                                letterSpacing: 2,
                              ),
                            ),
                            SizedBox(height: 4),
                            Text(
                              'Health & Vitality',
                              style: TextStyle(color: Colors.white, fontSize: 22, fontWeight: FontWeight.bold),
                            ),
                          ],
                        ),
                        IconButton(
                          onPressed: () => Navigator.pop(context),
                          icon: const Icon(LucideIcons.x, color: Colors.white38),
                        ),
                      ],
                    ),
                    const SizedBox(height: 24),
                    
                    // Date Selectors
                    Row(
                      children: [
                        Expanded(
                          child: _DateSelector(
                            label: 'Start Date',
                            date: _startDate,
                            onTap: () async {
                              final d = await showDatePicker(
                                context: context,
                                initialDate: _startDate,
                                firstDate: DateTime(2020),
                                lastDate: DateTime.now(),
                              );
                              if (d != null) {
                                setState(() => _startDate = d);
                                _syncData();
                              }
                            },
                          ),
                        ),
                        const SizedBox(width: 12),
                        Expanded(
                          child: _DateSelector(
                            label: 'End Date',
                            date: _endDate,
                            onTap: () async {
                              final d = await showDatePicker(
                                context: context,
                                initialDate: _endDate,
                                firstDate: DateTime(2020),
                                lastDate: DateTime.now(),
                              );
                              if (d != null) {
                                setState(() => _endDate = d);
                                _syncData();
                              }
                            },
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ),

          // Main List
          Expanded(
            child: historyStream.when(
              data: (items) {
                if (items.isEmpty) {
                  return Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(LucideIcons.activity, size: 48, color: Colors.white.withValues(alpha: 0.05)),
                        const SizedBox(height: 16),
                        Text(
                          'No biometrics synced',
                          style: TextStyle(color: Colors.white.withValues(alpha: 0.2), fontSize: 14, fontWeight: FontWeight.bold),
                        ),
                      ],
                    ),
                  );
                }

                final stats = _calculateStats(items);
                final stepsData = items.reversed.map((i) => i.steps.toDouble()).toList();

                return ListView(
                  padding: const EdgeInsets.all(24),
                  children: [
                    // Glanceable Cards
                    Row(
                      children: [
                        Expanded(child: _StatCard(label: 'Steps', value: stats['steps']!, icon: LucideIcons.footprints, color: const Color(AppConfig.emerald))),
                        const SizedBox(width: 8),
                        Expanded(child: _StatCard(label: 'Avg HR', value: stats['heart']!, icon: LucideIcons.heart, color: const Color(AppConfig.rose), suffix: ' BPM')),
                        const SizedBox(width: 8),
                        Expanded(child: _StatCard(label: 'Sleep', value: stats['sleep']!, icon: LucideIcons.moon, color: Colors.indigo, suffix: ' HRS')),
                      ],
                    ),
                    const SizedBox(height: 24),
                    
                    // Chart
                    _ActivityChart(data: stepsData, startDate: _startDate, endDate: _endDate),
                    const SizedBox(height: 32),

                    const Text(
                      'DAILY BREAKDOWN',
                      style: TextStyle(color: Colors.white24, fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5),
                    ),
                    const SizedBox(height: 12),
                    ...items.map((i) => _DailyMetricItem(metric: HealthMetric.fromDb(i))),
                  ],
                );
              },
              loading: () => const Center(child: CircularProgressIndicator(strokeWidth: 2)),
              error: (e, _) => Center(child: Text('Sync Error: $e', style: const TextStyle(color: Colors.red))),
            ),
          ),
        ],
      ),
    );
  }

  Map<String, String> _calculateStats(List<db.HealthMetric> items) {
    if (items.isEmpty) return {'steps': '0', 'heart': '--', 'sleep': '--'};
    
    int totalSteps = 0;
    int heartSum = 0;
    int heartCount = 0;
    double sleepSum = 0;
    int sleepCount = 0;

    for (var i in items) {
      totalSteps += i.steps;
      if (i.avgHeartRate != null) {
        heartSum += i.avgHeartRate!;
        heartCount++;
      }
      if (i.sleepHours != null) {
        sleepSum += i.sleepHours!;
        sleepCount++;
      }
    }

    return {
      'steps': NumberFormat('#,###', 'de_DE').format(totalSteps),
      'heart': heartCount > 0 ? (heartSum ~/ heartCount).toString() : '--',
      'sleep': sleepCount > 0 ? (sleepSum / sleepCount).toStringAsFixed(1) : '--',
    };
  }
}

class _DateSelector extends StatelessWidget {
  final String label;
  final DateTime date;
  final VoidCallback onTap;

  const _DateSelector({required this.label, required this.date, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(16),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        decoration: BoxDecoration(
          color: Colors.black.withValues(alpha: 0.3),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(label.toUpperCase(), style: const TextStyle(color: Colors.white24, fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1)),
            const SizedBox(height: 4),
            Row(
              children: [
                const Icon(LucideIcons.calendar, size: 12, color: Colors.blue),
                const SizedBox(width: 8),
                Text(DateFormat('MMM d, yyyy').format(date), style: const TextStyle(color: Colors.white, fontSize: 11, fontWeight: FontWeight.bold)),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _StatCard extends StatelessWidget {
  final String label;
  final String value;
  final String? suffix;
  final IconData icon;
  final Color color;

  const _StatCard({required this.label, required this.value, required this.icon, required this.color, this.suffix});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(24),
        border: Border.all(color: color.withValues(alpha: 0.15)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 20, color: color.withValues(alpha: 0.3)),
          const SizedBox(height: 12),
          Text(label.toUpperCase(), style: TextStyle(color: color.withValues(alpha: 0.6), fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1)),
          Row(
            crossAxisAlignment: CrossAxisAlignment.baseline,
            textBaseline: TextBaseline.alphabetic,
            children: [
              Text(value, style: TextStyle(color: color, fontSize: 18, fontWeight: FontWeight.w900, fontFamily: 'monospace')),
              if (suffix != null) Text(suffix!, style: TextStyle(color: color.withValues(alpha: 0.5), fontSize: 8, fontWeight: FontWeight.bold)),
            ],
          ),
        ],
      ),
    );
  }
}

class _ActivityChart extends StatelessWidget {
  final List<double> data;
  final DateTime startDate;
  final DateTime endDate;

  const _ActivityChart({required this.data, required this.startDate, required this.endDate});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.02),
        borderRadius: BorderRadius.circular(32),
        border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Row(
            children: [
              Icon(LucideIcons.trendingUp, size: 14, color: Colors.blue),
              SizedBox(width: 12),
              Text('ACTIVITY TREND', style: TextStyle(color: Colors.white24, fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
            ],
          ),
          const SizedBox(height: 24),
          SizedBox(
            height: 100,
            width: double.infinity,
            child: CustomPaint(
              painter: _LineChartPainter(data: data),
            ),
          ),
          const SizedBox(height: 12),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(DateFormat('MMM d').format(startDate), style: const TextStyle(color: Colors.white10, fontSize: 8, fontWeight: FontWeight.bold)),
              Text(DateFormat('MMM d').format(endDate), style: const TextStyle(color: Colors.white10, fontSize: 8, fontWeight: FontWeight.bold)),
            ],
          ),
        ],
      ),
    );
  }
}

class _LineChartPainter extends CustomPainter {
  final List<double> data;
  _LineChartPainter({required this.data});

  @override
  void paint(Canvas canvas, Size size) {
    if (data.length < 2) return;

    final maxVal = data.reduce(math.max).clamp(1.0, double.infinity);
    final stepX = size.width / (data.length - 1);
    
    final path = Path();
    final fillPath = Path();
    
    for (var i = 0; i < data.length; i++) {
      final x = i * stepX;
      final y = size.height - (data[i] / maxVal) * size.height;
      
      if (i == 0) {
        path.moveTo(x, y);
        fillPath.moveTo(x, size.height);
        fillPath.lineTo(x, y);
      } else {
        path.lineTo(x, y);
        fillPath.lineTo(x, y);
      }
      
      if (i == data.length - 1) {
        fillPath.lineTo(x, size.height);
        fillPath.close();
      }
    }

    final paint = Paint()
      ..color = Colors.blue
      ..style = PaintingStyle.stroke
      ..strokeWidth = 3
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;

    final fillPaint = Paint()
      ..shader = LinearGradient(
        begin: Alignment.topCenter,
        end: Alignment.bottomCenter,
        colors: [Colors.blue.withValues(alpha: 0.1), Colors.blue.withValues(alpha: 0)],
      ).createShader(Rect.fromLTWH(0, 0, size.width, size.height));

    canvas.drawPath(fillPath, fillPaint);
    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => true;
}

class _DailyMetricItem extends StatelessWidget {
  final HealthMetric metric;
  const _DailyMetricItem({required this.metric});

  @override
  Widget build(BuildContext context) {
    final date = DateTime.parse(metric.logDate);
    final updated = DateTime.parse(metric.updatedAt);
    
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.03),
        borderRadius: BorderRadius.circular(20),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(DateFormat('EEE, MMM d').format(date), style: const TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.bold)),
              Text('SYNCED ${DateFormat('HH:mm').format(updated)}', style: const TextStyle(color: Colors.white12, fontSize: 8, fontWeight: FontWeight.bold, letterSpacing: 0.5)),
            ],
          ),
          Row(
            children: [
              _MetricMini(label: 'Steps', value: NumberFormat('#,###', 'de_DE').format(metric.metrics.steps), color: const Color(AppConfig.emerald)),
              const SizedBox(width: 16),
              _MetricMini(label: 'Heart', value: metric.metrics.avgHeartRate?.toString() ?? '--', color: const Color(AppConfig.rose), suffix: 'BPM'),
              const SizedBox(width: 16),
              _MetricMini(label: 'Sleep', value: metric.metrics.sleepHours?.toStringAsFixed(1) ?? '--', color: Colors.indigo, suffix: 'H'),
            ],
          ),
        ],
      ),
    );
  }
}

class _MetricMini extends StatelessWidget {
  final String label;
  final String value;
  final Color color;
  final String? suffix;

  const _MetricMini({required this.label, required this.value, required this.color, this.suffix});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.end,
      children: [
        Text(label.toUpperCase(), style: const TextStyle(color: Colors.white10, fontSize: 7, fontWeight: FontWeight.w900)),
        Row(
          children: [
            Text(value, style: TextStyle(color: color.withValues(alpha: 0.8), fontSize: 11, fontWeight: FontWeight.bold, fontFamily: 'monospace')),
            if (suffix != null) Text(suffix!, style: TextStyle(color: color.withValues(alpha: 0.3), fontSize: 7, fontWeight: FontWeight.bold)),
          ],
        ),
      ],
    );
  }
}

final healthHistoryStreamProvider = StreamProvider.family<List<db.HealthMetric>, (DateTime, DateTime)>((ref, range) {
  return ref.watch(chatRepositoryProvider).watchHealthHistory(start: range.$1, end: range.$2);
});
