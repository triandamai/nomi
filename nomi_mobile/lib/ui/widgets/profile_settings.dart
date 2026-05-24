import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:nomi_mobile/providers/chat_provider.dart';
import 'package:nomi_mobile/core/config.dart';

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

    return Dialog(
      backgroundColor: const Color(AppConfig.deepSlate).withAlpha(242),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(32), side: const BorderSide(color: Colors.white10)),
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.all(24),
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
                      color: Colors.blue.withAlpha(25),
                      borderRadius: BorderRadius.circular(24),
                      border: Border.all(color: Colors.blue.withAlpha(51)),
                    ),
                    child: Center(
                      child: Text(
                        (authState.user?.displayName ?? 'U').substring(0, 1).toUpperCase(),
                        style: const TextStyle(color: Colors.blue, fontSize: 24, fontWeight: FontWeight.w900),
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
                          style: const TextStyle(color: Colors.white, fontSize: 18, fontWeight: FontWeight.bold),
                        ),
                        Text(
                          authState.user?.role?.toUpperCase() ?? 'USER',
                          style: const TextStyle(color: Colors.blue, fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1),
                        ),
                      ],
                    ),
                  ),
                  IconButton(
                    onPressed: () => Navigator.pop(context),
                    icon: const Icon(LucideIcons.x, color: Colors.white24, size: 18),
                  ),
                ],
              ),
              const SizedBox(height: 32),

              // Display Identity Section
              _buildSectionLabel(LucideIcons.user, 'Display Identity'),
              const SizedBox(height: 12),
              Container(
                decoration: BoxDecoration(
                  color: Colors.white.withAlpha(13),
                  borderRadius: BorderRadius.circular(20),
                  border: Border.all(color: Colors.white.withAlpha(25)),
                ),
                child: TextField(
                  controller: _nameController,
                  style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 14),
                  decoration: InputDecoration(
                    hintText: 'How should Nomi address you?',
                    hintStyle: TextStyle(color: Colors.white.withAlpha(51)),
                    border: InputBorder.none,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
                    suffixIcon: _nameController.text != authState.user?.displayName 
                      ? IconButton(
                          icon: _isUpdating 
                            ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2))
                            : const Icon(LucideIcons.save, color: Colors.blue, size: 18),
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
                  style: TextStyle(color: Colors.white.withAlpha(51), fontSize: 9, fontWeight: FontWeight.bold),
                ),
              ),

              const SizedBox(height: 32),

              // Active Session Section
              _buildSectionLabel(LucideIcons.messageSquare, 'Active Sessions'),
              const SizedBox(height: 12),
              Container(
                constraints: const BoxConstraints(maxHeight: 200),
                decoration: BoxDecoration(
                  color: Colors.white.withAlpha(5),
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
                        leading: const Icon(LucideIcons.hash, size: 14, color: Colors.purple),
                        title: Text(conv.name ?? 'Private Sandbox', style: const TextStyle(color: Colors.white70, fontSize: 12, fontWeight: FontWeight.bold)),
                        trailing: Container(
                          width: 6,
                          height: 6,
                          decoration: const BoxDecoration(color: Color(AppConfig.emerald), shape: BoxShape.circle),
                        ),
                      );
                    },
                  ),
                  loading: () => const Center(child: CircularProgressIndicator(strokeWidth: 2)),
                  error: (e, _) => const SizedBox.shrink(),
                ),
              ),
              const SizedBox(height: 32),

              // Danger Zone / Logout
              const Divider(color: Colors.white10),
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
                    backgroundColor: const Color(AppConfig.rose).withValues(alpha: 0.1),
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
                  ),
                  child: const Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(LucideIcons.logOut, size: 16, color: Color(AppConfig.rose)),
                      SizedBox(width: 12),
                      Text(
                        'SIGN OUT SESSION',
                        style: TextStyle(
                          color: Color(AppConfig.rose),
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
    );
  }

  Widget _buildSectionLabel(IconData icon, String label) {
    return Row(
      children: [
        Icon(icon, size: 12, color: Colors.white38),
        const SizedBox(width: 8),
        Text(
          label.toUpperCase(),
          style: const TextStyle(color: Colors.white38, fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5),
        ),
      ],
    );
  }
}
