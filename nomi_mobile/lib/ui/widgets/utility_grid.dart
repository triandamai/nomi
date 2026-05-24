import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';

class UtilityGridSheet extends ConsumerWidget {
  const UtilityGridSheet({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final authState = ref.watch(authProvider);
    final bool isAdmin = authState.user?.role == 'admin';
    final size = MediaQuery.of(context).size;
    final bool isLargeScreen = size.width >= 700;

    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40),
        child: Container(
          width: double.infinity,
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
                // Header
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'SYSTEM UTILITIES',
                          style: TextStyle(
                            color: const Color(AppConfig.emerald),
                            fontSize: isLargeScreen ? 10 : 12,
                            fontWeight: FontWeight.w900,
                            letterSpacing: 2,
                          ),
                        ),
                        const SizedBox(height: 4),
                        Text(
                          'Command Center',
                          style: TextStyle(
                            color: Colors.white, 
                            fontSize: isLargeScreen ? 18 : 22, 
                            fontWeight: FontWeight.bold
                          ),
                        ),
                      ],
                    ),
                    IconButton(
                      onPressed: () => Navigator.pop(context),
                      icon: Icon(LucideIcons.x, color: Colors.white38, size: isLargeScreen ? 20 : 24),
                    ),
                  ],
                ),
                const SizedBox(height: 32),

                // Grid
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
                        onTap: () {},
                      ),
                      _UtilityButton(
                        icon: LucideIcons.dollarSign,
                        label: 'Money Tracking',
                        color: const Color(AppConfig.emerald),
                        isLargeScreen: isLargeScreen,
                        onTap: () {},
                      ),
                      _UtilityButton(
                        icon: LucideIcons.heartPulse,
                        label: 'Health & Vitality',
                        color: const Color(AppConfig.rose),
                        isLargeScreen: isLargeScreen,
                        onTap: () {},
                      ),
                      _UtilityButton(
                        icon: LucideIcons.bookOpen,
                        label: 'System Blueprint',
                        color: Colors.amber,
                        isLargeScreen: isLargeScreen,
                        onTap: () {},
                      ),
                      _UtilityButton(
                        icon: LucideIcons.cpu,
                        label: 'Edge Plugins',
                        color: Colors.indigo,
                        isLargeScreen: isLargeScreen,
                        onTap: () {},
                      ),
                      _UtilityButton(
                        icon: LucideIcons.factory,
                        label: 'Factory Console',
                        color: const Color(AppConfig.emerald),
                        isLargeScreen: isLargeScreen,
                        onTap: () {},
                      ),
                      
                      if (isAdmin) ...[
                        _UtilityButton(
                          icon: LucideIcons.shieldAlert,
                          label: 'Guardrails',
                          color: Colors.red,
                          isLargeScreen: isLargeScreen,
                          onTap: () {},
                        ),
                        _UtilityButton(
                          icon: LucideIcons.user,
                          label: 'User Directory',
                          color: Colors.purple,
                          isLargeScreen: isLargeScreen,
                          onTap: () {},
                        ),
                      ],
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
