import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/chat_provider.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/ui/widgets/utility_grid.dart';
import 'package:nomi_mobile/ui/widgets/profile_settings.dart';
import 'package:nomi_mobile/ui/widgets/avatar.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;

class NomiSidebar extends ConsumerWidget {
  final bool isDrawer;
  const NomiSidebar({super.key, this.isDrawer = true});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final chatState = ref.watch(chatProvider);
    final conversationsAsync = ref.watch(conversationsStreamProvider);
    final authState = ref.watch(authProvider);
    final bool isAdmin = authState.user?.role == 'admin';

    // 🏷️ Mobile Mode: Full Discord Sidebar as a Drawer
    if (isDrawer) {
      return Drawer(
        width: 320,
        backgroundColor: Colors.transparent,
        child: _buildFullSidebar(context, ref, chatState, conversationsAsync, isAdmin, authState),
      );
    }

    // 🖥️ Large Screen Mode: Permanent Rail + Overlaid "Drawer" Pane (LIQUID GLASS)
    return Row(
      children: [
        _buildNavigationRail(context, ref, isAdmin, authState, chatState),
        if (chatState.isSidebarExpanded)
          // 🚀 Overlaid "Drawer" Effect: Premium Liquid Glass
          ClipRRect(
            child: BackdropFilter(
              filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40), // 🎯 Ultra-high liquid blur
              child: Container(
                width: 248,
                decoration: BoxDecoration(
                  gradient: LinearGradient(
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                    colors: [
                      const Color(0xFF1e293b).withValues(alpha: 0.4),
                      const Color(0xFF0f172a).withValues(alpha: 0.2),
                    ],
                  ),
                  border: const Border(
                    left: BorderSide(color: Colors.white10),
                    right: BorderSide(color: Colors.white10),
                  ),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withValues(alpha: 0.5),
                      blurRadius: 50,
                      offset: const Offset(10, 0),
                    )
                  ],
                ),
                child: _buildConversationPane(context, ref, chatState, conversationsAsync, withBackground: false),
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildFullSidebar(BuildContext context, WidgetRef ref, ChatState chatState, AsyncValue<List<db.Conversation>> conversationsAsync, bool isAdmin, AuthState authState) {
    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 25, sigmaY: 25), // 🎯 Deep glass ONLY for mobile drawer
        child: Row(
          children: [
            _buildNavigationRail(context, ref, isAdmin, authState, chatState),
            _buildConversationPane(context, ref, chatState, conversationsAsync),
          ],
        ),
      ),
    );
  }

  Widget _buildNavigationRail(BuildContext context, WidgetRef ref, bool isAdmin, AuthState authState, ChatState chatState) {
    final Widget rail = Container(
      width: 72,
      decoration: BoxDecoration(
        color: isDrawer ? const Color(0xFF0f172a).withValues(alpha: 0.45) : Colors.transparent,
        gradient: isDrawer ? null : LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [
            const Color(0xFF0f172a).withValues(alpha: 0.5),
            const Color(0xFF020617).withValues(alpha: 0.3),
          ],
        ),
        border: const Border(right: BorderSide(color: Colors.white10)),
      ),
      padding: const EdgeInsets.symmetric(vertical: 12),
      child: Column(
        children: [
          _buildLogo(),
          const SizedBox(height: 8),
          _buildSeparator(),
          const SizedBox(height: 12),
          // Sidebar Toggle (Only for Large Screen)
          if (!isDrawer)
            _SidebarActionIcon(
              icon: LucideIcons.messageSquare,
              color: chatState.isSidebarExpanded ? const Color(AppConfig.blue) : Colors.white24,
              onTap: () => ref.read(chatProvider.notifier).toggleSidebar(),
            ),
          const Spacer(),
          _buildBottomActions(context, ref, isAdmin, authState),
        ],
      ),
    );

    if (isDrawer) return rail;

    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40),
        child: rail,
      ),
    );
  }

  Widget _buildConversationPane(BuildContext context, WidgetRef ref, ChatState chatState, AsyncValue<List<db.Conversation>> conversationsAsync, {bool withBackground = true}) {
    return Container(
      width: 248, // Standard Discord Pane width
      color: withBackground ? const Color(0xFF111b21).withValues(alpha: 0.45) : Colors.transparent,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: conversationsAsync.when(
              data: (conversations) => ListView.builder(
                padding: const EdgeInsets.fromLTRB(12, 48, 12, 12),
                itemCount: conversations.length,
                itemBuilder: (context, index) {
                  final conv = conversations[index];
                  final isActive = chatState.activeConversationId == conv.id;
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 4),
                    child: Material(
                      color: Colors.transparent,
                      child: ListTile(
                        onTap: () {
                          ref.read(chatProvider.notifier).setActiveConversation(conv.id);
                          if (isDrawer) {
                            Navigator.pop(context);
                          } else if (chatState.isSidebarExpanded) {
                            ref.read(chatProvider.notifier).toggleSidebar();
                          }
                        },
                        selected: isActive,
                        selectedTileColor: Colors.blue.withValues(alpha: 0.1),
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                        leading: Container(
                          padding: const EdgeInsets.all(8),
                          decoration: BoxDecoration(
                            color: isActive ? Colors.blue : Colors.white.withValues(alpha: 0.05),
                            borderRadius: BorderRadius.circular(10),
                          ),
                          child: Icon(
                            LucideIcons.hash,
                            size: 16,
                            color: isActive ? Colors.white : Colors.white24,
                          ),
                        ),
                        title: Text(
                          conv.name ?? 'Private Sandbox',
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            color: isActive ? Colors.white : Colors.white70,
                            fontSize: 13,
                            fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
                          ),
                        ),
                        trailing: isActive ? const Icon(LucideIcons.messageSquare, size: 14, color: Colors.blue) : null,
                      ),
                    ),
                  );
                },
              ),
              loading: () => const Center(child: CircularProgressIndicator(strokeWidth: 2)),
              error: (e, _) => const Icon(LucideIcons.alertTriangle, color: Colors.red, size: 16),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildLogo() {
    return Container(
      width: 48,
      height: 48,
      decoration: BoxDecoration(
        color: const Color(0xFF3b82f6),
        borderRadius: BorderRadius.circular(16),
        boxShadow: [
          BoxShadow(
            color: const Color(0xFF3b82f6).withValues(alpha: 0.2),
            blurRadius: 10,
            offset: const Offset(0, 4),
          )
        ],
      ),
      child: const Center(
        child: Text(
          'N',
          style: TextStyle(color: Colors.white, fontSize: 20, fontWeight: FontWeight.w900),
        ),
      ),
    );
  }

  Widget _buildSeparator() {
    return Container(
      width: 32,
      height: 2,
      decoration: BoxDecoration(
        color: Colors.white10,
        borderRadius: BorderRadius.circular(1),
      ),
    );
  }

  Widget _buildBottomActions(BuildContext context, WidgetRef ref, bool isAdmin, AuthState authState) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(vertical: 12),
      child: Column(
        children: [
          if (isAdmin)
            _SidebarActionIcon(
              icon: LucideIcons.plus,
              color: Colors.blue,
              onTap: () {},
            ),
          const SizedBox(height: 12),
          _SidebarActionIcon(
            icon: LucideIcons.layoutGrid,
            color: const Color(AppConfig.emerald),
            onTap: () {
              if (isDrawer) Navigator.pop(context);
              showModalBottomSheet(
                context: context,
                isScrollControlled: true,
                backgroundColor: Colors.transparent,
                barrierColor: Colors.black.withValues(alpha: 0.5),
                builder: (context) => const UtilityGridSheet(),
              );
            },
          ),
          const SizedBox(height: 12),
          NomiAvatar(
            name: authState.user?.displayName ?? 'User',
            size: 48,
            onTap: () {
              if (isDrawer) Navigator.pop(context);
              showDialog(
                context: context,
                builder: (context) => const ProfileSettingsDialog(),
              );
            },
          ),
        ],
      ),
    );
  }
}

class _SidebarActionIcon extends StatelessWidget {
  final IconData icon;
  final Color color;
  final VoidCallback onTap;

  const _SidebarActionIcon({
    required this.icon,
    required this.color,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(24),
      child: Container(
        width: 48,
        height: 48,
        decoration: BoxDecoration(
          color: const Color(0xFF1e293b).withValues(alpha: 0.5),
          borderRadius: BorderRadius.circular(24),
        ),
        child: Icon(icon, size: 24, color: color.withValues(alpha: 0.8)),
      ),
    );
  }
}
