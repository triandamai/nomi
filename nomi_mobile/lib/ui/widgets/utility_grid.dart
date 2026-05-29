import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/navigation_provider.dart';
import 'package:nomi_mobile/ui/widgets/reminder_history.dart';
import 'package:nomi_mobile/ui/widgets/finance_history.dart';
import 'package:nomi_mobile/ui/widgets/health_history.dart';
import 'package:nomi_mobile/ui/widgets/blueprint_viewer.dart';
import 'package:nomi_mobile/ui/widgets/plugin_console.dart';
import 'package:nomi_mobile/ui/widgets/factory_console.dart';
import 'package:nomi_mobile/ui/widgets/user_directory.dart';
import 'package:nomi_mobile/ui/widgets/autonomous_history.dart';

class UtilityGridSheet extends ConsumerWidget {
  const UtilityGridSheet({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    final size = MediaQuery.of(context).size;
    final bool isLargeScreen = size.width >= 700;
    final nav = ref.read(navigationProvider.notifier);

    return ClipRRect(
      borderRadius: const BorderRadius.only(
        topLeft: Radius.circular(20),
        topRight: Radius.circular(20),
      ),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: size.height * 0.9),
          decoration: BoxDecoration(
            color: themeState.isDark 
              ? Color(themeState.slate950).withValues(alpha: 0.85) 
              : Color(themeState.bgHeader).withValues(alpha: 0.92),
            borderRadius: const BorderRadius.only(
              topLeft: Radius.circular(20),
              topRight: Radius.circular(20),
            ),
            border: Border.all(
              color: Color(themeState.borderMain).withValues(alpha: 0.25),
              width: 1.2,
            ),
          ),
          padding: EdgeInsets.symmetric(horizontal: 24, vertical: isLargeScreen ? 24 : 32),
          child: SafeArea(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'SYSTEM UTILITIES', 
                          style: TextStyle(
                            color: Color(themeState.accentColor), 
                            fontSize: 10, 
                            fontWeight: FontWeight.w900, 
                            letterSpacing: 2
                          )
                        ),
                        const SizedBox(height: 4),
                        Text(
                          'Command Center', 
                          style: TextStyle(
                            color: Color(themeState.textMain), 
                            fontSize: 22, 
                            fontWeight: FontWeight.bold
                          )
                        ),
                      ],
                    ),
                    IconButton(
                      onPressed: () => Navigator.pop(context),
                      icon: Icon(LucideIcons.x, color: Color(themeState.textMuted), size: isLargeScreen ? 20 : 24),
                    ),
                  ],
                ),
                const SizedBox(height: 32),

                Flexible(
                  child: GridView.count(
                    shrinkWrap: true,
                    crossAxisCount: isLargeScreen ? 3 : 2,
                    mainAxisSpacing: isLargeScreen ? 12 : 20,
                    crossAxisSpacing: isLargeScreen ? 12 : 20,
                    childAspectRatio: isLargeScreen ? 1.4 : 1.1,
                    children: [
                      _UtilityButton(
                        icon: LucideIcons.bell,
                        label: 'Reminders',
                        color: Colors.blue,
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          showModalBottomSheet(context: context, isScrollControlled: true, backgroundColor: Colors.transparent, builder: (context) => const ReminderHistorySheet());
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.activity,
                        label: 'Nomi Workflows',
                        color: Color(AppConfig.indigo),
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          showModalBottomSheet(context: context, isScrollControlled: true, backgroundColor: Colors.transparent, builder: (context) => const AutonomousHistorySheet());
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.dollarSign,
                        label: 'Money Tracking',
                        color: Color(AppConfig.emerald),
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          showModalBottomSheet(context: context, isScrollControlled: true, backgroundColor: Colors.transparent, builder: (context) => const FinanceHistorySheet());
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.heartPulse,
                        label: 'Health & Vitality',
                        color: Color(AppConfig.rose),
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          showModalBottomSheet(context: context, isScrollControlled: true, backgroundColor: Colors.transparent, builder: (context) => const HealthHistorySheet());
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.bookOpen,
                        label: 'System Blueprint',
                        color: Colors.amber,
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          showModalBottomSheet(context: context, isScrollControlled: true, backgroundColor: Colors.transparent, builder: (context) => const BlueprintViewerSheet());
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.cpu,
                        label: 'Edge Plugins',
                        color: Colors.indigo,
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          showModalBottomSheet(context: context, isScrollControlled: true, backgroundColor: Colors.transparent, builder: (context) => const PluginConsoleSheet());
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.brain,
                        label: 'Reinforcement',
                        color: Colors.blue,
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          nav.navigateTo(MainView.reinforcement);
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.shieldCheck,
                        label: 'Guardrails',
                        color: Color(AppConfig.emerald),
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          nav.navigateTo(MainView.guardrails);
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.puzzle,
                        label: 'System Skills',
                        color: Color(AppConfig.indigo),
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          nav.navigateTo(MainView.skills);
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.lineChart,
                        label: 'Monitor',
                        color: Colors.blue,
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          nav.navigateTo(MainView.monitor);
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.users,
                        label: 'User Directory',
                        color: Color(AppConfig.blue),
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          showModalBottomSheet(context: context, isScrollControlled: true, backgroundColor: Colors.transparent, builder: (context) => const UserDirectorySheet());
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.database,
                        label: 'Storage Monitor',
                        color: Colors.purple,
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          nav.navigateTo(MainView.storage);
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.factory,
                        label: 'Factory Console',
                        color: Color(AppConfig.emerald),
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          showModalBottomSheet(context: context, isScrollControlled: true, backgroundColor: Colors.transparent, builder: (context) => const FactoryConsoleSheet());
                        },
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 24),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _UtilityButton extends ConsumerWidget {
  final IconData icon;
  final String label;
  final Color color;
  final bool isLargeScreen;
  final VoidCallback onTap;

  const _UtilityButton({
    required this.icon,
    required this.label,
    required this.color,
    required this.isLargeScreen,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    return Material(
      color: Color(themeState.textMain).withValues(alpha: 0.05),
      borderRadius: BorderRadius.circular(isLargeScreen ? 16 : 24),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(isLargeScreen ? 16 : 24),
        child: Container(
          padding: EdgeInsets.all(isLargeScreen ? 12 : 20),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(isLargeScreen ? 16 : 24),
            border: Border.all(color: color.withValues(alpha: 0.2)),
          ),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Container(
                padding: EdgeInsets.all(isLargeScreen ? 8 : 16),
                decoration: BoxDecoration(
                  color: color.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(isLargeScreen ? 12 : 20),
                ),
                child: Icon(icon, color: color, size: isLargeScreen ? 24 : 32),
              ),
              const SizedBox(height: 12),
              Text(
                label.toUpperCase(),
                textAlign: TextAlign.center,
                style: TextStyle(
                  color: Color(themeState.textMain),
                  fontSize: isLargeScreen ? 8 : 10,
                  fontWeight: FontWeight.w900,
                  letterSpacing: 1,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
