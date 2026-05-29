import 'dart:ui' show ImageFilter;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:nomi_mobile/providers/chat_provider.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
import 'package:nomi_mobile/core/localization/i18n.dart';

class ProfileSettingsDialog extends ConsumerStatefulWidget {
  const ProfileSettingsDialog({super.key});

  @override
  ConsumerState<ProfileSettingsDialog> createState() => _ProfileSettingsDialogState();
}

class _ProfileSettingsDialogState extends ConsumerState<ProfileSettingsDialog> {
  late TextEditingController _nameController;
  bool _isUpdating = false;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController(text: ref.read(authProvider).user?.displayName ?? '');
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final authState = ref.watch(authProvider);
    final conversationsAsync = ref.watch(conversationsStreamProvider);
    final themeState = ref.watch(themeProvider);

    return Dialog(
      backgroundColor: Colors.transparent,
      elevation: 0,
      insetPadding: const EdgeInsets.symmetric(horizontal: 20, vertical: 24),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(20),
        child: BackdropFilter(
          filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
          child: Container(
            width: double.infinity,
            padding: const EdgeInsets.all(24),
            decoration: BoxDecoration(
              color: themeState.isDark
                  ? Color(themeState.slate950).withAlpha(190)
                  : Color(themeState.bgHeader).withAlpha(220),
              borderRadius: BorderRadius.circular(20),
              border: Border.all(color: Color(themeState.borderMain).withAlpha(100)),
            ),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Identity Header
              Row(
                children: [
                  Container(
                    width: 64,
                    height: 64,
                    decoration: BoxDecoration(
                      color: Color(themeState.primaryColor).withAlpha(25),
                      borderRadius: BorderRadius.circular(24),
                      border: Border.all(color: Color(themeState.primaryColor).withAlpha(51)),
                    ),
                    child: Center(
                      child: Text(
                        (authState.user?.displayName ?? 'U').substring(0, 1).toUpperCase(),
                        style: TextStyle(color: Color(themeState.primaryColor), fontSize: 24, fontWeight: FontWeight.w900),
                      ),
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          authState.user?.displayName ?? 'Anonymous User',
                          style: TextStyle(color: Color(themeState.textMain), fontSize: 18, fontWeight: FontWeight.bold),
                        ),
                        Text(
                          authState.user?.role?.toUpperCase() ?? 'USER',
                          style: TextStyle(color: Color(themeState.primaryColor), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1),
                        ),
                      ],
                    ),
                  ),
                  IconButton(
                    onPressed: () => Navigator.pop(context),
                    icon: Icon(LucideIcons.x, color: Color(themeState.textMuted).withAlpha(80), size: 18),
                  ),
                ],
              ),
              const SizedBox(height: 32),

              // Display Identity Section
              _buildSectionLabel(themeState, LucideIcons.user, 'display_name'.tr(ref)),
              const SizedBox(height: 12),
              Container(
                decoration: BoxDecoration(
                  color: Color(themeState.textMain).withAlpha(13),
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: Color(themeState.textMain).withAlpha(25)),
                ),
                child: TextField(
                  controller: _nameController,
                  style: TextStyle(color: Color(themeState.textMain), fontWeight: FontWeight.bold, fontSize: 14),
                  decoration: InputDecoration(
                    hintText: 'How should Nomi address you?',
                    hintStyle: TextStyle(color: Color(themeState.textMain).withAlpha(51)),
                    border: InputBorder.none,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
                    suffixIcon: _nameController.text != authState.user?.displayName 
                      ? IconButton(
                          icon: _isUpdating 
                            ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2))
                            : Icon(LucideIcons.save, color: Color(themeState.primaryColor), size: 18),
                          onPressed: () async {
                             if (!mounted) return;
                             setState(() => _isUpdating = true);
                             final success = await ref.read(authProvider.notifier).updateProfile(_nameController.text);
                             if (mounted) {
                               ScaffoldMessenger.of(context).showSnackBar(
                                 SnackBar(content: Text(success ? 'Profile updated' : 'Update failed')),
                               );
                               setState(() => _isUpdating = false);
                             }
                          },
                        )
                      : null,
                  ),
                ),
              ),
              const SizedBox(height: 8),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 4),
                child: Text(
                  'Identity Source: ${authState.user?.externalId}',
                  style: TextStyle(color: Color(themeState.textMuted), fontSize: 9, fontWeight: FontWeight.bold),
                ),
              ),

              const SizedBox(height: 32),

              // Active Session Section
              _buildSectionLabel(themeState, LucideIcons.messageSquare, 'Active Sessions'),
              const SizedBox(height: 12),
              Container(
                constraints: const BoxConstraints(maxHeight: 140),
                decoration: BoxDecoration(
                  color: Color(themeState.textMain).withAlpha(5),
                  borderRadius: BorderRadius.circular(20),
                ),
                child: conversationsAsync.when(
                  data: (conversations) => ListView.builder(
                    shrinkWrap: true,
                    itemCount: conversations.length,
                    itemBuilder: (context, index) {
                      final conv = conversations[index];
                      return ListTile(
                        dense: true,
                        leading: Icon(LucideIcons.hash, size: 14, color: Color(themeState.primaryColor)),
                        title: Text(conv.name ?? 'Private Sandbox', style: TextStyle(color: Color(themeState.textMain).withAlpha(200), fontSize: 12, fontWeight: FontWeight.bold)),
                        trailing: Container(
                          width: 6,
                          height: 6,
                          decoration: BoxDecoration(color: Color(themeState.accentColor), shape: BoxShape.circle),
                        ),
                      );
                    },
                  ),
                  loading: () => const Center(child: CircularProgressIndicator(strokeWidth: 2)),
                  error: (e, _) => const SizedBox.shrink(),
                ),
              ),

              const SizedBox(height: 32),

              // Theme Settings Section
              _buildSectionLabel(themeState, LucideIcons.palette, 'Interface Style & Themes'),
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
                decoration: BoxDecoration(
                  color: Color(themeState.textMain).withAlpha(12),
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: Color(themeState.borderMain).withAlpha(60)),
                ),
                child: DropdownButtonHideUnderline(
                  child: DropdownButtonFormField<NomiTheme>(
                    value: themeState,
                    dropdownColor: themeState.isDark ? Color(themeState.slate950) : Color(themeState.bgHeader),
                    borderRadius: BorderRadius.circular(24),
                    icon: Icon(LucideIcons.chevronDown, color: Color(themeState.primaryColor), size: 18),
                    decoration: const InputDecoration(
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                    ),
                    onChanged: (newTheme) {
                      if (newTheme != null) {
                        ref.read(themeProvider.notifier).setTheme(newTheme);
                      }
                    },
                    selectedItemBuilder: (BuildContext context) {
                      return NomiTheme.values.map((theme) {
                        return Row(
                          children: [
                            Container(
                              width: 10,
                              height: 10,
                              decoration: BoxDecoration(
                                color: Color(theme.primaryColor),
                                shape: BoxShape.circle,
                              ),
                            ),
                            const SizedBox(width: 6),
                            Container(
                              width: 10,
                              height: 10,
                              decoration: BoxDecoration(
                                color: Color(theme.accentColor),
                                shape: BoxShape.circle,
                              ),
                            ),
                            const SizedBox(width: 10),
                            Text(
                              theme.name,
                              style: TextStyle(
                                color: Color(themeState.textMain),
                                fontSize: 13,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                          ],
                        );
                      }).toList();
                    },
                    items: NomiTheme.values.map((theme) {
                      final isSelected = theme == themeState;
                      return DropdownMenuItem<NomiTheme>(
                        value: theme,
                        child: Container(
                          padding: const EdgeInsets.symmetric(vertical: 4),
                          child: Row(
                            children: [
                              Container(
                                width: 10,
                                height: 10,
                                decoration: BoxDecoration(
                                  color: Color(theme.primaryColor),
                                  shape: BoxShape.circle,
                                  boxShadow: [
                                    BoxShadow(
                                      color: Color(theme.primaryColor).withAlpha(80),
                                      blurRadius: 4,
                                    )
                                  ],
                                ),
                              ),
                              const SizedBox(width: 6),
                              Container(
                                width: 10,
                                height: 10,
                                decoration: BoxDecoration(
                                  color: Color(theme.accentColor),
                                  shape: BoxShape.circle,
                                ),
                              ),
                              const SizedBox(width: 12),
                              Expanded(
                                child: Text(
                                  theme.name,
                                  style: TextStyle(
                                    color: isSelected ? Color(themeState.primaryColor) : Color(themeState.textMain),
                                    fontSize: 13,
                                    fontWeight: isSelected ? FontWeight.w900 : FontWeight.bold,
                                  ),
                                ),
                              ),
                              if (isSelected)
                                Icon(LucideIcons.check, color: Color(themeState.primaryColor), size: 14),
                            ],
                          ),
                        ),
                      );
                    }).toList(),
                  ),
                ),
              ),

              // Language Settings Section
              const SizedBox(height: 24),
              _buildSectionLabel(themeState, LucideIcons.languages, 'select_language'.tr(ref)),
              const SizedBox(height: 12),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
                decoration: BoxDecoration(
                  color: Color(themeState.textMain).withAlpha(12),
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: Color(themeState.borderMain).withAlpha(60)),
                ),
                child: DropdownButtonHideUnderline(
                  child: DropdownButtonFormField<String>(
                    value: ref.watch(localeProvider),
                    dropdownColor: themeState.isDark ? Color(themeState.slate950) : Color(themeState.bgHeader),
                    borderRadius: BorderRadius.circular(24),
                    icon: Icon(LucideIcons.chevronDown, color: Color(themeState.primaryColor), size: 18),
                    decoration: const InputDecoration(
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                    ),
                    onChanged: (newLocale) {
                      if (newLocale != null) {
                        ref.read(localeProvider.notifier).state = newLocale;
                      }
                    },
                    items: [
                      DropdownMenuItem<String>(
                        value: 'en',
                        child: Text(
                          'English (US)',
                          style: TextStyle(color: Color(themeState.textMain), fontSize: 13, fontWeight: FontWeight.bold),
                        ),
                      ),
                      DropdownMenuItem<String>(
                        value: 'id',
                        child: Text(
                          'Bahasa Indonesia',
                          style: TextStyle(color: Color(themeState.textMain), fontSize: 13, fontWeight: FontWeight.bold),
                        ),
                      ),
                    ],
                  ),
                ),
              ),

              const SizedBox(height: 32),

              // Danger Zone / Logout
              Divider(color: Color(themeState.borderMain)),
              const SizedBox(height: 16),
              SizedBox(
                width: double.infinity,
                child: TextButton(
                  onPressed: () {
                    Navigator.pop(context);
                    ref.read(authProvider.notifier).logout();
                  },
                  style: TextButton.styleFrom(
                    padding: const EdgeInsets.symmetric(vertical: 16),
                    backgroundColor: Color(themeState.accentColor).withAlpha(25),
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
                  ),
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(LucideIcons.logOut, size: 16, color: Color(themeState.accentColor)),
                      const SizedBox(width: 12),
                      Text(
                        'logout'.tr(ref).toUpperCase(),
                        style: TextStyle(
                          color: Color(themeState.accentColor),
                          fontSize: 11,
                          fontWeight: FontWeight.w900,
                          letterSpacing: 1.5,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(height: 24),
            ],
          ),
        ),
      ),
    ),
    ),
    );
  }

  Widget _buildSectionLabel(NomiTheme themeState, IconData icon, String label) {
    return Row(
      children: [
        Icon(icon, size: 12, color: Color(themeState.textMuted)),
        const SizedBox(width: 8),
        Text(
          label.toUpperCase(),
          style: TextStyle(color: Color(themeState.textMuted), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5),
        ),
      ],
    );
  }
}
