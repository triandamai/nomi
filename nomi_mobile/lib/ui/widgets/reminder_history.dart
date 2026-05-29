import 'dart:ui' show ImageFilter;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/data/models/reminder.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;
import 'package:intl/intl.dart';

class ReminderHistorySheet extends ConsumerStatefulWidget {
  const ReminderHistorySheet({super.key});

  @override
  ConsumerState<ReminderHistorySheet> createState() => _ReminderHistorySheetState();
}

class _ReminderHistorySheetState extends ConsumerState<ReminderHistorySheet> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(chatRepositoryProvider).syncReminders();
    });
  }

  @override
  Widget build(BuildContext context) {
    final themeState = ref.watch(themeProvider);
    final remindersStream = ref.watch(remindersStreamProvider);
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
                      'TECHNICAL TASKS',
                      style: TextStyle(
                        color: Color(themeState.accentColor),
                        fontSize: 10,
                        fontWeight: FontWeight.w900,
                        letterSpacing: 2,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'Your Reminders',
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

          // List
          Expanded(
            child: remindersStream.when(
              data: (items) {
                if (items.isEmpty) {
                  return Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(LucideIcons.bell, size: 48, color: Color(themeState.textMuted).withValues(alpha: 0.1)),
                        const SizedBox(height: 16),
                        Text(
                          'No upcoming reminders',
                          style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.4), fontSize: 14, fontWeight: FontWeight.bold),
                        ),
                      ],
                    ),
                  );
                }
                return ListView.builder(
                  padding: const EdgeInsets.symmetric(horizontal: 24),
                  itemCount: items.length,
                  itemBuilder: (context, index) {
                    return _ReminderItem(reminder: Reminder.fromDb(items[index]));
                  },
                );
              },
              loading: () => const Center(child: CircularProgressIndicator(strokeWidth: 2)),
              error: (e, _) => Center(child: Text('Error: $e', style: const TextStyle(color: Colors.red))),
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

class _ReminderItem extends ConsumerWidget {
  final Reminder reminder;
  const _ReminderItem({required this.reminder});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    final due = DateTime.parse(reminder.dueAt);
    final isCompleted = reminder.status == 'completed';
    
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Color(themeState.textMain).withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              _buildTypeIcon(),
              const SizedBox(width: 8),
              Text(
                reminder.taskType ?? 'REMINDER',
                style: TextStyle(
                  color: Color(themeState.textMuted).withValues(alpha: 0.6),
                  fontSize: 8,
                  fontWeight: FontWeight.w900,
                  letterSpacing: 1.5,
                ),
              ),
              const Spacer(),
              if (reminder.frequency != null && reminder.frequency != 'once')
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Color(themeState.textMain).withValues(alpha: 0.05),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    reminder.frequency!.toUpperCase(),
                    style: TextStyle(color: Color(themeState.textMuted), fontSize: 7, fontWeight: FontWeight.bold),
                  ),
                ),
            ],
          ),
          const SizedBox(height: 12),
          Text(
            reminder.content,
            style: TextStyle(color: Color(themeState.textMain), fontSize: 14, fontWeight: FontWeight.w600, height: 1.4),
          ),
          const SizedBox(height: 16),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: themeState.isDark ? Colors.black26 : Color(themeState.textMain).withValues(alpha: 0.05),
                  borderRadius: BorderRadius.circular(6),
                ),
                child: Row(
                  children: [
                    Icon(LucideIcons.calendar, size: 10, color: Color(themeState.primaryColor)),
                    const SizedBox(width: 6),
                    Text(
                      '${DateFormat('MMM d').format(due)} · ${DateFormat('HH:mm').format(due)}',
                      style: TextStyle(color: Color(themeState.primaryColor), fontSize: 10, fontWeight: FontWeight.bold, fontFamily: 'monospace'),
                    ),
                  ],
                ),
              ),
              Text(
                reminder.status.toUpperCase(),
                style: TextStyle(
                  color: isCompleted ? Color(themeState.accentColor) : Colors.amber,
                  fontSize: 8,
                  fontWeight: FontWeight.w900,
                  letterSpacing: 1,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildTypeIcon() {
    IconData icon;
    Color color;
    
    switch (reminder.taskType) {
      case 'SEND_DM':
        icon = LucideIcons.messageSquare;
        color = Colors.blue;
        break;
      case 'TRIGGER_AGENT':
        icon = LucideIcons.cpu;
        color = Colors.purple;
        break;
      default:
        icon = LucideIcons.bell;
        color = Colors.amber;
    }
    
    return Icon(icon, size: 12, color: color);
  }
}

final remindersStreamProvider = StreamProvider<List<db.Reminder>>((ref) {
  return ref.watch(chatRepositoryProvider).watchReminders();
});
