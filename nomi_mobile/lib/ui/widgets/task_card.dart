import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/core/config.dart';

class TaskCard extends ConsumerStatefulWidget {
  final String refId;
  final bool collapsed;

  const TaskCard({
    super.key,
    required this.refId,
    this.collapsed = true,
  });

  @override
  ConsumerState<TaskCard> createState() => _TaskCardState();
}

class _TaskCardState extends ConsumerState<TaskCard> {
  Map<String, dynamic>? _data;
  bool _isLoading = true;
  bool _isCollapsed = true;
  bool _expandedLogs = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _isCollapsed = widget.collapsed;
    _fetchDetail();
  }

  Future<void> _fetchDetail() async {
    try {
      final response = await ref.read(apiClientProvider).dio.get('/tasks/${widget.refId}/timeline');
      if (!mounted) return;
      if (response.data != null && response.data['meta']['code'] == 200) {
        setState(() {
          _data = response.data['data'];
          _isLoading = false;
        });
      } else {
        setState(() {
          _error = "Task not found";
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

  Color _getStatusColor(String? status) {
    switch (status?.toLowerCase()) {
      case 'running':
        return const Color(AppConfig.amber);
      case 'completed':
        return const Color(AppConfig.emerald);
      case 'failed':
        return const Color(AppConfig.rose);
      case 'cancelled':
        return Colors.white24;
      default:
        return Colors.white38;
    }
  }

  Color _getEventMarkerColor(String? eventType) {
    switch (eventType?.toLowerCase()) {
      case 'step_start':
        return const Color(AppConfig.amber);
      case 'tool_execution':
        return const Color(AppConfig.blue);
      case 'human_response':
        return const Color(AppConfig.indigo);
      case 'outbound_msg':
        return const Color(AppConfig.rose);
      default:
        return const Color(AppConfig.emerald);
    }
  }

  IconData _getEventIcon(String? eventType) {
    switch (eventType?.toLowerCase()) {
      case 'human_response':
        return LucideIcons.userCheck;
      case 'tool_execution':
        return LucideIcons.messageSquareCode;
      case 'outbound_msg':
        return LucideIcons.send;
      default:
        return LucideIcons.cpu;
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return Container(
        margin: const EdgeInsets.symmetric(vertical: 8),
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: const Color(AppConfig.deepSlate).withValues(alpha: 0.6),
          borderRadius: BorderRadius.circular(20),
          border: Border.all(color: Colors.white10),
        ),
        child: const Center(
          child: SizedBox(
            width: 20,
            height: 20,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
        ),
      );
    }

    if (_error != null || _data == null) {
      return const SizedBox.shrink(); // Hide if failed to load
    }

    final String title = _data!['title'] ?? 'Nomi Workflow';
    final String globalGoal = _data!['global_goal'] ?? 'No description';
    final String status = _data!['status'] ?? 'pending';
    final int currentStepIndex = _data!['current_step_index'] ?? 0;
    final List<dynamic> checkpoints = _data!['checkpoints'] ?? [];
    final List<dynamic> logs = _data!['logs'] ?? [];
    final int? cumulativeTokens = _data!['cumulative_tokens'];

    final Color statusColor = _getStatusColor(status);

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        color: const Color(0xFF0b1329).withValues(alpha: 0.8),
        borderRadius: BorderRadius.circular(24),
        border: Border.all(color: Colors.white.withValues(alpha: 0.08)),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.3),
            blurRadius: 15,
            offset: const Offset(0, 8),
          ),
        ],
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Card Header
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              decoration: BoxDecoration(
                color: Colors.white.withValues(alpha: 0.02),
                border: Border(bottom: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
              ),
              child: Row(
                children: [
                  Container(
                    width: 32,
                    height: 32,
                    decoration: BoxDecoration(
                      color: const Color(AppConfig.indigo).withValues(alpha: 0.15),
                      borderRadius: BorderRadius.circular(10),
                      border: Border.all(color: const Color(AppConfig.indigo).withValues(alpha: 0.35)),
                    ),
                    child: const Icon(LucideIcons.activity, size: 16, color: Color(AppConfig.indigo)),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            const Text(
                              'NOMI WORKFLOW',
                              style: TextStyle(
                                color: Colors.white70,
                                fontSize: 9,
                                fontWeight: FontWeight.w900,
                                letterSpacing: 1.2,
                              ),
                            ),
                            if (cumulativeTokens != null) ...[
                              const SizedBox(width: 6),
                              Container(
                                padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                                decoration: BoxDecoration(
                                  color: Colors.white.withValues(alpha: 0.05),
                                  borderRadius: BorderRadius.circular(4),
                                  border: Border.all(color: Colors.white.withValues(alpha: 0.1)),
                                ),
                                child: Text(
                                  '⚡ $cumulativeTokens',
                                  style: const TextStyle(
                                    color: Colors.white70,
                                    fontSize: 8,
                                    fontWeight: FontWeight.bold,
                                  ),
                                ),
                              ),
                            ],
                          ],
                        ),
                        const SizedBox(height: 2),
                        Text(
                          title,
                          style: const TextStyle(
                            color: Colors.white,
                            fontSize: 13,
                            fontWeight: FontWeight.bold,
                          ),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 8),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                    decoration: BoxDecoration(
                      color: statusColor.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(12),
                      border: Border.all(color: statusColor.withValues(alpha: 0.2)),
                    ),
                    child: Text(
                      status.toUpperCase().replaceAll('_', ' '),
                      style: TextStyle(
                        color: statusColor,
                        fontSize: 8,
                        fontWeight: FontWeight.bold,
                        letterSpacing: 0.5,
                      ),
                    ),
                  ),
                ],
              ),
            ),

            // Collapsible state
            if (_isCollapsed)
              InkWell(
                onTap: () => setState(() => _isCollapsed = false),
                child: Container(
                  width: double.infinity,
                  padding: const EdgeInsets.symmetric(vertical: 10),
                  color: Colors.white.withValues(alpha: 0.01),
                  child: const Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Text(
                        'SHOW TASK DETAILS',
                        style: TextStyle(
                          color: Colors.white38,
                          fontSize: 9,
                          fontWeight: FontWeight.w900,
                          letterSpacing: 1.2,
                        ),
                      ),
                      SizedBox(width: 6),
                      Icon(LucideIcons.chevronDown, size: 12, color: Colors.white38),
                    ],
                  ),
                ),
              )
            else ...[
              Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    // Global Goal / Objective
                    Container(
                      width: double.infinity,
                      padding: const EdgeInsets.all(12),
                      decoration: BoxDecoration(
                        color: Colors.white.withValues(alpha: 0.02),
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: Colors.white.withValues(alpha: 0.04)),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          const Text(
                            'GLOBAL OBJECTIVE',
                            style: TextStyle(
                              color: Colors.white38,
                              fontSize: 8,
                              fontWeight: FontWeight.w900,
                              letterSpacing: 1.0,
                            ),
                          ),
                          const SizedBox(height: 6),
                          Text(
                            globalGoal,
                            style: const TextStyle(
                              color: Colors.white70,
                              fontSize: 12,
                              height: 1.4,
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 16),

                    // Checkpoints
                    if (checkpoints.isNotEmpty) ...[
                      const Text(
                        'CHECKPOINTS PLAN',
                        style: TextStyle(
                          color: Colors.white38,
                          fontSize: 8,
                          fontWeight: FontWeight.w900,
                          letterSpacing: 1.0,
                        ),
                      ),
                      const SizedBox(height: 8),
                      ListView.separated(
                        shrinkWrap: true,
                        physics: const NeverScrollableScrollPhysics(),
                        itemCount: checkpoints.length,
                        separatorBuilder: (context, index) => const SizedBox(height: 8),
                        itemBuilder: (context, index) {
                          final cp = checkpoints[index];
                          final int cpStepIndex = cp['step_index'] ?? 0;
                          final String objective = cp['action_objective'] ?? '';
                          final String cpStatus = cp['status'] ?? 'pending';

                          final bool isActive = cpStepIndex == currentStepIndex && status.toLowerCase() == 'running';
                          final bool isCompleted = cpStatus == 'completed' || cpStepIndex < currentStepIndex;
                          final bool isFailed = cpStatus == 'failed';

                          Color itemBgColor = Colors.white.withValues(alpha: 0.01);
                          Color itemBorderColor = Colors.white.withValues(alpha: 0.03);
                          Widget statusWidget = const Icon(LucideIcons.circle, size: 14, color: Colors.white24);
                          Color labelColor = Colors.white38;

                          if (isActive) {
                            itemBgColor = const Color(AppConfig.amber).withValues(alpha: 0.05);
                            itemBorderColor = const Color(AppConfig.amber).withValues(alpha: 0.3);
                            statusWidget = const SizedBox(
                              width: 14,
                              height: 14,
                              child: CircularProgressIndicator(
                                strokeWidth: 1.5,
                                valueColor: AlwaysStoppedAnimation<Color>(Color(AppConfig.amber)),
                              ),
                            );
                            labelColor = const Color(AppConfig.amber);
                          } else if (isCompleted) {
                            itemBgColor = const Color(AppConfig.emerald).withValues(alpha: 0.03);
                            itemBorderColor = const Color(AppConfig.emerald).withValues(alpha: 0.15);
                            statusWidget = const Icon(LucideIcons.checkCircle2, size: 14, color: Color(AppConfig.emerald));
                            labelColor = const Color(AppConfig.emerald).withValues(alpha: 0.7);
                          } else if (isFailed) {
                            itemBgColor = const Color(AppConfig.rose).withValues(alpha: 0.05);
                            itemBorderColor = const Color(AppConfig.rose).withValues(alpha: 0.3);
                            statusWidget = const Icon(LucideIcons.alertCircle, size: 14, color: Color(AppConfig.rose));
                            labelColor = const Color(AppConfig.rose);
                          }

                          return Container(
                            padding: const EdgeInsets.all(12),
                            decoration: BoxDecoration(
                              color: itemBgColor,
                              borderRadius: BorderRadius.circular(14),
                              border: Border.all(color: itemBorderColor),
                            ),
                            child: Row(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Padding(
                                  padding: const EdgeInsets.only(top: 2),
                                  child: statusWidget,
                                ),
                                const SizedBox(width: 10),
                                Expanded(
                                  child: Column(
                                    crossAxisAlignment: CrossAxisAlignment.start,
                                    children: [
                                      Text(
                                        'STEP ${cpStepIndex + 1}',
                                        style: TextStyle(
                                          color: labelColor,
                                          fontSize: 8,
                                          fontWeight: FontWeight.w900,
                                          letterSpacing: 0.8,
                                        ),
                                      ),
                                      const SizedBox(height: 2),
                                      Text(
                                        objective,
                                        style: const TextStyle(
                                          color: Colors.white,
                                          fontSize: 12,
                                          fontWeight: FontWeight.bold,
                                        ),
                                      ),
                                    ],
                                  ),
                                ),
                              ],
                            ),
                          );
                        },
                      ),
                      const SizedBox(height: 16),
                    ],

                    // Logs timeline
                    if (logs.isNotEmpty) ...[
                      const Divider(color: Colors.white10),
                      const SizedBox(height: 8),
                      InkWell(
                        onTap: () => setState(() => _expandedLogs = !_expandedLogs),
                        child: Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Row(
                              children: [
                                const Icon(LucideIcons.cpu, size: 14, color: Color(AppConfig.indigo)),
                                const SizedBox(width: 8),
                                Text(
                                  'ACTION TIMELINE (${logs.length})',
                                  style: const TextStyle(
                                    color: Colors.white38,
                                    fontSize: 8,
                                    fontWeight: FontWeight.w900,
                                    letterSpacing: 1.0,
                                  ),
                                ),
                              ],
                            ),
                            Icon(
                              _expandedLogs ? LucideIcons.chevronUp : LucideIcons.chevronDown,
                              size: 14,
                              color: Colors.white38,
                            ),
                          ],
                        ),
                      ),
                      if (_expandedLogs) ...[
                        const SizedBox(height: 12),
                        ListView.builder(
                          shrinkWrap: true,
                          physics: const NeverScrollableScrollPhysics(),
                          itemCount: logs.length,
                          itemBuilder: (context, index) {
                            final log = logs[index];
                            final String eventType = log['event_type'] ?? '';
                            final String logContent = log['log_content'] ?? '';
                            final Color markerColor = _getEventMarkerColor(eventType);
                            final IconData eventIcon = _getEventIcon(eventType);

                            return IntrinsicHeight(
                              child: Row(
                                crossAxisAlignment: CrossAxisAlignment.stretch,
                                children: [
                                  // Timeline line & marker column
                                  SizedBox(
                                    width: 24,
                                    child: Column(
                                      children: [
                                        Container(
                                          width: 8,
                                          height: 8,
                                          decoration: BoxDecoration(
                                            color: markerColor,
                                            shape: BoxShape.circle,
                                            border: Border.all(color: const Color(0xFF0b1329), width: 1.5),
                                          ),
                                        ),
                                        Expanded(
                                          child: Container(
                                            width: 1.5,
                                            color: index == logs.length - 1
                                                ? Colors.transparent
                                                : Colors.white10,
                                          ),
                                        ),
                                      ],
                                    ),
                                  ),
                                  const SizedBox(width: 8),
                                  // Log details
                                  Expanded(
                                    child: Padding(
                                      padding: const EdgeInsets.only(bottom: 16),
                                      child: Column(
                                        crossAxisAlignment: CrossAxisAlignment.start,
                                        children: [
                                          Row(
                                            children: [
                                              Icon(eventIcon, size: 10, color: markerColor),
                                              const SizedBox(width: 4),
                                              Text(
                                                eventType.toUpperCase().replaceAll('_', ' '),
                                                style: TextStyle(
                                                  color: markerColor,
                                                  fontSize: 8,
                                                  fontWeight: FontWeight.w900,
                                                  letterSpacing: 0.8,
                                                ),
                                              ),
                                            ],
                                          ),
                                          const SizedBox(height: 4),
                                          Text(
                                            logContent,
                                            style: const TextStyle(
                                              color: Colors.white70,
                                              fontSize: 11,
                                              height: 1.4,
                                            ),
                                          ),
                                        ],
                                      ),
                                    ),
                                  ),
                                ],
                              ),
                            );
                          },
                        ),
                      ],
                    ],
                  ],
                ),
              ),

              // Collapse button
              InkWell(
                onTap: () => setState(() => _isCollapsed = true),
                child: Container(
                  width: double.infinity,
                  padding: const EdgeInsets.symmetric(vertical: 10),
                  decoration: BoxDecoration(
                    color: Colors.white.withValues(alpha: 0.01),
                    border: Border(top: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
                  ),
                  child: const Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Text(
                        'HIDE TASK DETAILS',
                        style: TextStyle(
                          color: Colors.white38,
                          fontSize: 9,
                          fontWeight: FontWeight.w900,
                          letterSpacing: 1.2,
                        ),
                      ),
                      SizedBox(width: 6),
                      Icon(LucideIcons.chevronUp, size: 12, color: Colors.white38),
                    ],
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
