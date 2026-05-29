import 'dart:ui' show ImageFilter;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/ui/widgets/task_card.dart';
import 'package:intl/intl.dart';

class AutonomousHistorySheet extends ConsumerStatefulWidget {
  const AutonomousHistorySheet({super.key});

  @override
  ConsumerState<AutonomousHistorySheet> createState() => _AutonomousHistorySheetState();
}

class _AutonomousHistorySheetState extends ConsumerState<AutonomousHistorySheet> {
  List<dynamic> _tasks = [];
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _fetchTasks();
  }

  Future<void> _fetchTasks() async {
    try {
      final response = await ref.read(apiClientProvider).dio.get('/tasks');
      if (!mounted) return;
      if (response.data != null && response.data['meta']['code'] == 200) {
        setState(() {
          _tasks = response.data['data'] as List<dynamic>;
          _isLoading = false;
        });
      } else {
        setState(() {
          _error = "Failed to load workflows";
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

  Color _getStatusColor(NomiTheme themeState, String? status) {
    switch (status?.toLowerCase()) {
      case 'running':
        return const Color(AppConfig.amber);
      case 'completed':
        return const Color(AppConfig.emerald);
      case 'failed':
        return const Color(AppConfig.rose);
      case 'cancelled':
        return Color(themeState.textMuted).withValues(alpha: 0.4);
      default:
        return Color(themeState.textMuted).withValues(alpha: 0.6);
    }
  }

  void _showTaskDetail(String taskId) {
    final themeState = ref.read(themeProvider);
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => ClipRRect(
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(20),
          topRight: Radius.circular(20),
        ),
        child: BackdropFilter(
          filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
          child: Container(
            padding: EdgeInsets.only(
              bottom: MediaQuery.of(context).viewInsets.bottom,
            ),
            decoration: BoxDecoration(
              color: themeState.isDark 
                ? Color(themeState.slate950).withValues(alpha: 0.85) 
                : Color(themeState.bgHeader).withValues(alpha: 0.92),
              border: Border.all(
                color: Color(themeState.borderMain).withValues(alpha: 0.25),
                width: 1.2,
              ),
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(20),
                topRight: Radius.circular(20),
              ),
            ),
        child: SafeArea(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Header Drag Handle
              Container(
                margin: const EdgeInsets.symmetric(vertical: 12),
                width: 40,
                height: 4,
                decoration: BoxDecoration(
                  color: Color(themeState.textMuted).withValues(alpha: 0.2),
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
              // Task details container
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: TaskCard(refId: taskId, collapsed: false),
              ),
              const SizedBox(height: 16),
            ],
          ),
        ),
      ),
      ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final themeState = ref.watch(themeProvider);
    final size = MediaQuery.of(context).size;
    final bool isLargeScreen = size.width >= 700;

    return ClipRRect(
      borderRadius: const BorderRadius.only(
        topLeft: Radius.circular(20),
        topRight: Radius.circular(20),
      ),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(
            maxHeight: size.height * 0.85,
          ),
          decoration: BoxDecoration(
            color: themeState.isDark 
              ? Color(themeState.slate950).withValues(alpha: 0.85) 
              : Color(themeState.bgHeader).withValues(alpha: 0.92),
            border: Border.all(
              color: Color(themeState.borderMain).withValues(alpha: 0.25),
              width: 1.2,
            ),
            borderRadius: const BorderRadius.only(
              topLeft: Radius.circular(20),
              topRight: Radius.circular(20),
            ),
          ),
          child: Column(
        children: [
          // Header
          Padding(
            padding: const EdgeInsets.all(24),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'AUTONOMOUS WORKFLOWS',
                      style: TextStyle(
                        color: Color(themeState.primaryColor),
                        fontSize: 10,
                        fontWeight: FontWeight.w900,
                        letterSpacing: 2,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'Nomi Workflows',
                      style: TextStyle(
                        color: Color(themeState.textMain),
                        fontSize: isLargeScreen ? 24 : 20,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ),
                IconButton(
                  onPressed: () => Navigator.pop(context),
                  icon: Icon(LucideIcons.x, color: Color(themeState.textMuted)),
                ),
              ],
            ),
          ),

          // Content
          Expanded(
            child: _isLoading
                ? const Center(child: CircularProgressIndicator(strokeWidth: 2))
                : _error != null
                    ? Center(
                        child: Text(
                          'Error: $_error',
                          style: const TextStyle(color: Color(AppConfig.rose)),
                        ),
                      )
                    : _tasks.isEmpty
                        ? Center(
                            child: Column(
                              mainAxisAlignment: MainAxisAlignment.center,
                              children: [
                                Icon(LucideIcons.activity, size: 48, color: Color(themeState.textMuted).withValues(alpha: 0.1)),
                                const SizedBox(height: 16),
                                Text(
                                  'No active workflows',
                                  style: TextStyle(
                                    color: Color(themeState.textMuted).withValues(alpha: 0.4),
                                    fontSize: 14,
                                    fontWeight: FontWeight.bold,
                                  ),
                                ),
                              ],
                            ),
                          )
                        : ListView.builder(
                            padding: const EdgeInsets.symmetric(horizontal: 24),
                            itemCount: _tasks.length,
                            itemBuilder: (context, index) {
                              final task = _tasks[index];
                              final String taskId = task['id'] ?? '';
                              final String title = task['title'] ?? 'Workflow';
                              final String globalGoal = task['global_goal'] ?? '';
                              final String status = task['status'] ?? 'pending';
                              final int currentStepIndex = task['current_step_index'] ?? 0;
                              final DateTime createdAt = DateTime.parse(task['created_at']);
                              final Color statusColor = _getStatusColor(themeState, status);

                              return Container(
                                margin: const EdgeInsets.only(bottom: 12),
                                decoration: BoxDecoration(
                                  color: Color(themeState.textMain).withValues(alpha: 0.03),
                                  borderRadius: BorderRadius.circular(20),
                                  border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.3)),
                                ),
                                child: InkWell(
                                  onTap: () => _showTaskDetail(taskId),
                                  borderRadius: BorderRadius.circular(20),
                                  child: Padding(
                                    padding: const EdgeInsets.all(16),
                                    child: Column(
                                      crossAxisAlignment: CrossAxisAlignment.start,
                                      children: [
                                        Row(
                                          children: [
                                            Icon(LucideIcons.activity, size: 12, color: Color(themeState.primaryColor)),
                                            const SizedBox(width: 8),
                                            Text(
                                              'STEP ${currentStepIndex + 1}',
                                              style: TextStyle(
                                                color: Color(themeState.textMuted),
                                                fontSize: 8,
                                                fontWeight: FontWeight.w900,
                                                letterSpacing: 1.5,
                                              ),
                                            ),
                                            const Spacer(),
                                            Container(
                                              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                                              decoration: BoxDecoration(
                                                color: statusColor.withValues(alpha: 0.1),
                                                borderRadius: BorderRadius.circular(8),
                                                border: Border.all(color: statusColor.withValues(alpha: 0.2)),
                                              ),
                                              child: Text(
                                                status.toUpperCase().replaceAll('_', ' '),
                                                style: TextStyle(
                                                  color: statusColor,
                                                  fontSize: 7,
                                                  fontWeight: FontWeight.bold,
                                                ),
                                              ),
                                            ),
                                          ],
                                        ),
                                        const SizedBox(height: 12),
                                        Text(
                                          title,
                                          style: TextStyle(
                                            color: Color(themeState.textMain),
                                            fontSize: 14,
                                            fontWeight: FontWeight.w600,
                                            height: 1.4,
                                          ),
                                        ),
                                        const SizedBox(height: 6),
                                        Text(
                                          globalGoal,
                                          maxLines: 2,
                                          overflow: TextOverflow.ellipsis,
                                          style: TextStyle(
                                            color: Color(themeState.textMuted),
                                            fontSize: 11,
                                            height: 1.4,
                                          ),
                                        ),
                                        const SizedBox(height: 16),
                                        Row(
                                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                                          children: [
                                            Container(
                                              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                                              decoration: BoxDecoration(
                                                color: Color(themeState.textMain).withValues(alpha: 0.05),
                                                borderRadius: BorderRadius.circular(6),
                                              ),
                                              child: Row(
                                                children: [
                                                  Icon(LucideIcons.calendar, size: 10, color: Color(themeState.primaryColor)),
                                                  const SizedBox(width: 6),
                                                  Text(
                                                    '${DateFormat('MMM d').format(createdAt)} · ${DateFormat('HH:mm').format(createdAt)}',
                                                    style: TextStyle(
                                                      color: Color(themeState.primaryColor),
                                                      fontSize: 10,
                                                      fontWeight: FontWeight.bold,
                                                      fontFamily: 'monospace',
                                                    ),
                                                  ),
                                                ],
                                              ),
                                            ),
                                            Row(
                                              children: [
                                                Text(
                                                  'VIEW DETAILS',
                                                  style: TextStyle(
                                                    color: Color(themeState.textMuted),
                                                    fontSize: 8,
                                                    fontWeight: FontWeight.w900,
                                                    letterSpacing: 1,
                                                  ),
                                                ),
                                                const SizedBox(width: 4),
                                                Icon(LucideIcons.chevronRight, size: 10, color: Color(themeState.textMuted)),
                                              ],
                                            ),
                                          ],
                                        ),
                                      ],
                                    ),
                                  ),
                                ),
                              );
                            },
                          ),
          ),
          const SizedBox(height: 24),
        ],
      ),
    ),
    ),
    );
  }
}
