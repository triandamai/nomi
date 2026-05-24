import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:intl/intl.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/core/config.dart';

class ReminderCard extends ConsumerStatefulWidget {
  final String refId;

  const ReminderCard({super.key, required this.refId});

  @override
  ConsumerState<ReminderCard> createState() => _ReminderCardState();
}

class _ReminderCardState extends ConsumerState<ReminderCard> {
  Map<String, dynamic>? _data;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _fetchDetail();
  }

  Future<void> _fetchDetail() async {
    try {
      final response = await ref.read(apiClientProvider).dio.get('/reminders/${widget.refId}');
      if (!mounted) return;
      if (response.data != null && response.data['meta']['code'] == 200) {
        setState(() {
          _data = response.data['data'];
          _isLoading = false;
        });
      } else {
        setState(() {
          _error = "Not found";
          _isLoading = false;
        });
      }
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return Container(
        margin: const EdgeInsets.symmetric(vertical: 8),
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: const Color(AppConfig.deepSlate).withAlpha(153),
          borderRadius: BorderRadius.circular(20),
          border: Border.all(color: Colors.white10),
        ),
        child: const Center(child: SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2))),
      );
    }

    if (_error != null || _data == null) {
      return const SizedBox.shrink(); // Hide if failed to load
    }

    final String content = _data!['content'] ?? 'No description';
    final String status = _data!['status'] ?? 'pending';
    final DateTime dueAt = DateTime.parse(_data!['due_at']);
    final String frequency = _data!['frequency'] ?? 'once';

    final timeFormat = DateFormat.Hm();
    final dateFormat = DateFormat('MMM d');

    const Color emeraldColor = Color(AppConfig.emerald);

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        color: const Color(0xFF0f172a).withAlpha(153),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: emeraldColor.withAlpha(77)),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withAlpha(51),
            blurRadius: 10,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Header
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              decoration: BoxDecoration(
                color: emeraldColor.withAlpha(25),
                border: Border(bottom: BorderSide(color: Colors.white.withAlpha(13))),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Row(
                    children: [
                      Icon(LucideIcons.bell, size: 12, color: emeraldColor),
                      SizedBox(width: 8),
                      Text(
                        'ACTIVE REMINDER',
                        style: TextStyle(
                          color: emeraldColor,
                          fontSize: 9,
                          fontWeight: FontWeight.w900,
                          letterSpacing: 1.2,
                        ),
                      ),
                    ],
                  ),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                    decoration: BoxDecoration(
                      color: emeraldColor.withAlpha(51),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: Text(
                      status.toUpperCase(),
                      style: const TextStyle(color: emeraldColor, fontSize: 8, fontWeight: FontWeight.bold),
                    ),
                  ),
                ],
              ),
            ),

            // Content
            Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    content,
                    style: const TextStyle(color: Colors.white, fontSize: 14, fontWeight: FontWeight.w500, height: 1.5),
                  ),
                  const SizedBox(height: 16),
                  Row(
                    children: [
                      _buildInfoItem(LucideIcons.clock, timeFormat.format(dueAt)),
                      const SizedBox(width: 16),
                      _buildInfoItem(LucideIcons.calendar, dateFormat.format(dueAt)),
                    ],
                  ),
                  if (frequency != 'once') ...[
                    const SizedBox(height: 12),
                    Row(
                      children: [
                        const Icon(LucideIcons.checkCircle2, size: 12, color: emeraldColor),
                        const SizedBox(width: 6),
                        Text(
                          'Repeats: $frequency',
                          style: TextStyle(color: emeraldColor.withAlpha(153), fontSize: 9, fontWeight: FontWeight.bold),
                        ),
                      ],
                    ),
                  ],
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildInfoItem(IconData icon, String text) {
    const Color emeraldColor = Color(AppConfig.emerald);
    return Row(
      children: [
        Icon(icon, size: 12, color: emeraldColor),
        const SizedBox(width: 6),
        Text(
          text,
          style: const TextStyle(color: Colors.white70, fontSize: 10, fontWeight: FontWeight.bold, letterSpacing: 0.5),
        ),
      ],
    );
  }
}
