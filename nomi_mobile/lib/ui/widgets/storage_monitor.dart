import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/core/utils/storage_monitor.dart';
import 'package:nomi_mobile/core/utils/formatter.dart';

class StorageMonitorSheet extends ConsumerStatefulWidget {
  const StorageMonitorSheet({super.key});

  @override
  ConsumerState<StorageMonitorSheet> createState() => _StorageMonitorSheetState();
}

class _StorageMonitorSheetState extends ConsumerState<StorageMonitorSheet> {
  Map<String, dynamic>? _metrics;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _refresh();
  }

  Future<void> _refresh() async {
    setState(() => _isLoading = true);
    final metrics = await StorageMonitor.getStorageMetrics();
    if (mounted) {
      setState(() {
        _metrics = metrics;
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: MediaQuery.of(context).size.height * 0.9),
          decoration: BoxDecoration(
            gradient: LinearGradient(
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
              colors: [const Color(AppConfig.deepSlate).withValues(alpha: 0.7), const Color(0xFF1e293b).withValues(alpha: 0.4)],
            ),
            border: const Border(top: BorderSide(color: Colors.white10)),
          ),
          padding: const EdgeInsets.all(24),
          child: _isLoading 
            ? const Center(child: CircularProgressIndicator())
            : Column(
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      const Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('STORAGE', style: TextStyle(color: Color(AppConfig.blue), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
                          SizedBox(height: 4),
                          Text('System Health', style: TextStyle(color: Colors.white, fontSize: 22, fontWeight: FontWeight.bold)),
                        ],
                      ),
                      IconButton(onPressed: () => Navigator.pop(context), icon: const Icon(LucideIcons.x, color: Colors.white38)),
                    ],
                  ),
                  const SizedBox(height: 32),
                  _buildMetricCard('Database Size', _metrics?['dbSize'] ?? 0),
                  _buildMetricCard('Cache Size', _metrics?['cacheSize'] ?? 0),
                ],
              ),
        ),
      ),
    );
  }

  Widget _buildMetricCard(String label, int size) {
    return Container(
      margin: const EdgeInsets.only(bottom: 16),
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(color: Colors.white.withValues(alpha: 0.03), borderRadius: BorderRadius.circular(20), border: Border.all(color: Colors.white10)),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold)),
          Text(Formatter.formatBytes(size), style: const TextStyle(color: Color(AppConfig.blue), fontFamily: 'monospace')),
        ],
      ),
    );
  }
}
