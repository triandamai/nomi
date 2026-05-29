import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/chat_provider.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:nomi_mobile/providers/navigation_provider.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
import 'package:nomi_mobile/ui/widgets/utility_grid.dart';
import 'package:nomi_mobile/ui/widgets/profile_settings.dart';
import 'package:nomi_mobile/ui/widgets/avatar.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;
import 'package:nomi_mobile/core/localization/i18n.dart';

class NomiSidebar extends ConsumerWidget {
  final bool isDrawer;
  const NomiSidebar({super.key, this.isDrawer = true});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final chatState = ref.watch(chatProvider);
    final themeState = ref.watch(themeProvider);
    final conversationsAsync = ref.watch(conversationsStreamProvider);
    final authState = ref.watch(authProvider);
    final bool isAdmin = authState.user?.role == 'admin';

    if (isDrawer) {
      return Drawer(
        width: 320,
        backgroundColor: Colors.transparent,
        child: _buildFullSidebar(context, ref, chatState, conversationsAsync, isAdmin, authState),
      );
    }

    return Row(
      children: [
        _buildNavigationRail(context, ref, isAdmin, authState, chatState, themeState),
        if (chatState.isSidebarExpanded)
          ClipRRect(
            child: BackdropFilter(
              filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
              child: Container(
                width: 248,
                decoration: BoxDecoration(
                  color: themeState.isDark 
                    ? Color(themeState.slate950).withValues(alpha: 0.6) 
                    : Color(themeState.textMain).withValues(alpha: 0.08),
                  border: Border(
                    left: BorderSide(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
                    right: BorderSide(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
                  ),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withValues(alpha: 0.05),
                      blurRadius: 30,
                      offset: const Offset(5, 0),
                    )
                  ],
                ),
                child: _buildConversationPane(context, ref, chatState, conversationsAsync, themeState, withBackground: false),
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildFullSidebar(BuildContext context, WidgetRef ref, ChatState chatState, AsyncValue<List<db.Conversation>> conversationsAsync, bool isAdmin, AuthState authState) {
    final themeState = ref.watch(themeProvider);
    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: Row(
          children: [
            _buildNavigationRail(context, ref, isAdmin, authState, chatState, themeState),
            _buildConversationPane(context, ref, chatState, conversationsAsync, themeState),
          ],
        ),
      ),
    );
  }

  Widget _buildNavigationRail(BuildContext context, WidgetRef ref, bool isAdmin, AuthState authState, ChatState chatState, NomiTheme themeState) {
    final Widget rail = Container(
      width: 72,
      decoration: BoxDecoration(
        color: isDrawer 
          ? (themeState.isDark 
              ? Color(themeState.slate950).withValues(alpha: 0.6) 
              : Color(themeState.textMain).withValues(alpha: 0.08))
          : Colors.transparent,
        gradient: isDrawer ? null : LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [
            Color(themeState.bgHeader).withValues(alpha: 0.5),
            Color(themeState.bgMain).withValues(alpha: 0.3),
          ],
        ),
        border: Border(right: BorderSide(color: Color(themeState.borderMain).withValues(alpha: 0.5))),
      ),
      padding: const EdgeInsets.symmetric(vertical: 12),
      child: SafeArea(
        bottom: false,
        child: Column(
          children: [
            _buildLogo(ref, themeState),
            const SizedBox(height: 8),
            _buildSeparator(themeState),
            const SizedBox(height: 12),
            if (!isDrawer)
              _SidebarActionIcon(
                icon: LucideIcons.messageSquare,
                color: chatState.isSidebarExpanded 
                  ? Color(themeState.primaryColor) 
                  : Color(themeState.textMuted),
                onTap: () => ref.read(chatProvider.notifier).toggleSidebar(),
              ),
            const Spacer(),
            _buildBottomActions(context, ref, isAdmin, authState, themeState),
          ],
        ),
      ),
    );

    if (isDrawer) return rail;

    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: rail,
      ),
    );
  }

  Widget _buildConversationPane(BuildContext context, WidgetRef ref, ChatState chatState, AsyncValue<List<db.Conversation>> conversationsAsync, NomiTheme themeState, {bool withBackground = true}) {
    return Container(
      width: 248,
      color: withBackground 
        ? (themeState.isDark 
            ? Color(themeState.slate950).withValues(alpha: 0.6) 
            : Color(themeState.textMain).withValues(alpha: 0.08))
        : Colors.transparent,
      child: SafeArea(
        bottom: false,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: conversationsAsync.when(
                data: (conversations) => ListView.builder(
                  padding: const EdgeInsets.fromLTRB(12, 12, 12, 12),
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
                          ref.read(navigationProvider.notifier).navigateTo(MainView.chat);
                          if (isDrawer) {
                            Navigator.pop(context);
                          } else if (chatState.isSidebarExpanded) {
                            ref.read(chatProvider.notifier).toggleSidebar();
                          }
                        },
                        selected: isActive,
                        selectedTileColor: Color(themeState.primaryColor).withValues(alpha: 0.1),
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                        leading: Container(
                          padding: const EdgeInsets.all(8),
                          decoration: BoxDecoration(
                            color: isActive 
                              ? Color(themeState.primaryColor) 
                              : Color(themeState.textMain).withValues(alpha: 0.05),
                            borderRadius: BorderRadius.circular(10),
                          ),
                          child: Icon(
                            LucideIcons.hash,
                            size: 16,
                            color: isActive ? Colors.white : Color(themeState.textMuted),
                          ),
                        ),
                        title: Text(
                          conv.name ?? 'private_sandbox'.tr(ref),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            color: isActive ? Color(themeState.primaryColor) : Color(themeState.textMain),
                            fontSize: 13,
                            fontWeight: isActive ? FontWeight.w900 : FontWeight.w500,
                          ),
                        ),
                        trailing: isActive ? Icon(LucideIcons.messageSquare, size: 14, color: Color(themeState.primaryColor)) : null,
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
    ),
  );
}

  Widget _buildLogo(WidgetRef ref, NomiTheme themeState) {
    return GestureDetector(
      onTap: () => ref.read(navigationProvider.notifier).navigateTo(MainView.chat),
      child: Container(
        width: 48,
        height: 48,
        decoration: BoxDecoration(
          color: Color(themeState.primaryColor),
          borderRadius: BorderRadius.circular(16),
          boxShadow: [
            BoxShadow(
              color: Color(themeState.primaryColor).withValues(alpha: 0.2),
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
      ),
    );
  }

  Widget _buildSeparator(NomiTheme themeState) {
    return Container(
      width: 32,
      height: 2,
      decoration: BoxDecoration(
        color: Color(themeState.borderMain),
        borderRadius: BorderRadius.circular(1),
      ),
    );
  }

  Widget _buildBottomActions(BuildContext context, WidgetRef ref, bool isAdmin, AuthState authState, NomiTheme themeState) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(vertical: 12),
      child: Column(
        children: [
          if (isAdmin)
            _SidebarActionIcon(
              icon: LucideIcons.plus,
              color: Color(themeState.primaryColor),
              onTap: () {},
            ),
          const SizedBox(height: 12),
          _SidebarActionIcon(
            icon: LucideIcons.layoutGrid,
            color: Color(themeState.accentColor),
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

class _SidebarActionIcon extends ConsumerWidget {
  final IconData icon;
  final Color color;
  final VoidCallback onTap;

  const _SidebarActionIcon({
    required this.icon,
    required this.color,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(24),
      child: Container(
        width: 48,
        height: 48,
        decoration: BoxDecoration(
          color: themeState.isDark 
            ? Color(themeState.slate900).withValues(alpha: 0.5)
            : Color(themeState.textMain).withValues(alpha: 0.05),
          borderRadius: BorderRadius.circular(24),
        ),
        child: Icon(icon, size: 24, color: color),
      ),
    );
  }
}
