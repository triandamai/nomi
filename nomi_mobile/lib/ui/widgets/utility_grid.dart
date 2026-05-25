import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/ui/widgets/reminder_history.dart';
import 'package:nomi_mobile/ui/widgets/finance_history.dart';
import 'package:nomi_mobile/ui/widgets/health_history.dart';
import 'package:nomi_mobile/ui/widgets/blueprint_viewer.dart';
import 'package:nomi_mobile/ui/widgets/plugin_console.dart';
import 'package:nomi_mobile/ui/widgets/factory_console.dart';
import 'package:nomi_mobile/ui/widgets/user_directory.dart';
import 'package:nomi_mobile/ui/pages/storage_page.dart';
import 'package:nomi_mobile/ui/pages/reinforcement_page.dart';
import 'package:nomi_mobile/ui/pages/monitor_page.dart';
import 'package:nomi_mobile/ui/pages/guardrail_page.dart';
import 'package:nomi_mobile/ui/pages/skills_page.dart';

class UtilityGridSheet extends ConsumerWidget {
  const UtilityGridSheet({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final size = MediaQuery.of(context).size;
    final bool isLargeScreen = size.width >= 700;

    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: size.height * 0.9),
          decoration: BoxDecoration(
            gradient: LinearGradient(
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
              colors: [
                const Color(AppConfig.deepSlate).withValues(alpha: 0.7),
                const Color(0xFF1e293b).withValues(alpha: 0.4),
              ],
            ),
            borderRadius: BorderRadius.zero,
            border: const Border(top: BorderSide(color: Colors.white10)),
          ),
          padding: EdgeInsets.symmetric(horizontal: 24, vertical: isLargeScreen ? 24 : 32),
          child: SafeArea(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('SYSTEM UTILITIES', style: TextStyle(color: Color(AppConfig.emerald), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
                        SizedBox(height: 4),
                        Text('Command Center', style: TextStyle(color: Colors.white, fontSize: 22, fontWeight: FontWeight.bold)),
                      ],
                    ),
                    IconButton(
                      onPressed: () => Navigator.pop(context),
                      icon: Icon(LucideIcons.x, color: Colors.white38, size: isLargeScreen ? 20 : 24),
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
                        icon: LucideIcons.dollarSign,
                        label: 'Money Tracking',
                        color: const Color(AppConfig.emerald),
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          showModalBottomSheet(context: context, isScrollControlled: true, backgroundColor: Colors.transparent, builder: (context) => const FinanceHistorySheet());
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.heartPulse,
                        label: 'Health & Vitality',
                        color: const Color(AppConfig.rose),
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
                          Navigator.push(context, MaterialPageRoute(builder: (context) => const ReinforcementPage()));
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.shieldCheck,
                        label: 'Guardrails',
                        color: const Color(AppConfig.emerald),
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          Navigator.push(context, MaterialPageRoute(builder: (context) => const GuardrailPage()));
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.puzzle,
                        label: 'System Skills',
                        color: const Color(AppConfig.indigo),
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          Navigator.push(context, MaterialPageRoute(builder: (context) => const SkillsPage()));
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.lineChart,
                        label: 'Monitor',
                        color: Colors.blue,
                        isLargeScreen: isLargeScreen,
                        onTap: () {
                          Navigator.pop(context);
                          Navigator.push(context, MaterialPageRoute(builder: (context) => const MonitorPage()));
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.users,
                        label: 'User Directory',
                        color: const Color(AppConfig.blue),
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
                          Navigator.push(context, MaterialPageRoute(builder: (context) => const StoragePage()));
                        },
                      ),
                      _UtilityButton(
                        icon: LucideIcons.factory,
                        label: 'Factory Console',
                        color: const Color(AppConfig.emerald),
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

class _UtilityButton extends StatelessWidget {
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
  Widget build(BuildContext context) {
    return Material(
      color: Colors.white.withAlpha(13),
      borderRadius: BorderRadius.circular(isLargeScreen ? 16 : 24),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(isLargeScreen ? 16 : 24),
        child: Container(
          padding: EdgeInsets.all(isLargeScreen ? 12 : 20),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(isLargeScreen ? 16 : 24),
            border: Border.all(color: color.withAlpha(51)),
          ),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Container(
                padding: EdgeInsets.all(isLargeScreen ? 8 : 16),
                decoration: BoxDecoration(
                  color: color.withAlpha(25),
                  borderRadius: BorderRadius.circular(isLargeScreen ? 12 : 20),
                ),
                child: Icon(icon, color: color, size: isLargeScreen ? 24 : 32),
              ),
              const SizedBox(height: 12),
              Text(
                label.toUpperCase(),
                textAlign: TextAlign.center,
                style: TextStyle(
                  color: Colors.white.withAlpha(204),
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
