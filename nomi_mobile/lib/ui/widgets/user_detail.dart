import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/data/models/user_detail.dart';

class UserDetailSheet extends ConsumerStatefulWidget {
  final String userId;
  const UserDetailSheet({super.key, required this.userId});

  @override
  ConsumerState<UserDetailSheet> createState() => _UserDetailSheetState();
}

class _UserDetailSheetState extends ConsumerState<UserDetailSheet> {
  UserDetail? _user;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _fetchDetail();
  }

  Future<void> _fetchDetail() async {
    final detail = await ref.read(chatRepositoryProvider).getUserDetail(widget.userId);
    if (mounted) {
      setState(() {
        _user = detail;
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final themeState = ref.watch(themeProvider);
    return ClipRRect(
      borderRadius: const BorderRadius.only(
        topLeft: Radius.circular(20),
        topRight: Radius.circular(20),
      ),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: MediaQuery.of(context).size.height * 0.9),
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
          padding: const EdgeInsets.all(24),
          child: _isLoading
              ? const Center(child: CircularProgressIndicator())
              : _user == null
                  ? Center(child: Text('User not found', style: TextStyle(color: Color(themeState.textMain))))
                  : SingleChildScrollView(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          _buildHeader(themeState),
                          const SizedBox(height: 24),
                          _buildSection(themeState, 'CONVERSATIONS', _user!.conversations.map((c) => c.title ?? 'Untitled').toList()),
                          const SizedBox(height: 24),
                          _buildSection(themeState, 'CHANNELS', _user!.channels.map((c) => '${c.channelType}: ${c.conversationTitle ?? 'No Title'}').toList()),
                        ],
                      ),
                    ),
        ),
      ),
    );
  }

  Widget _buildHeader(NomiTheme themeState) {
    return Row(
      children: [
        CircleAvatar(
          radius: 30, 
          backgroundColor: Color(themeState.primaryColor).withValues(alpha: 0.1),
          child: Text(
            _user!.displayName?[0].toUpperCase() ?? '?', 
            style: TextStyle(color: Color(themeState.primaryColor), fontWeight: FontWeight.bold)
          )
        ),
        const SizedBox(width: 16),
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(_user!.displayName ?? 'Unknown', style: TextStyle(color: Color(themeState.textMain), fontSize: 18, fontWeight: FontWeight.bold)),
            Text(_user!.email ?? '', style: TextStyle(color: Color(themeState.textMuted))),
          ],
        ),
        const Spacer(),
        IconButton(onPressed: () => Navigator.pop(context), icon: Icon(LucideIcons.x, color: Color(themeState.textMuted))),
      ],
    );
  }

  Widget _buildSection(NomiTheme themeState, String title, List<String> items) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: TextStyle(color: Color(themeState.primaryColor), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
        const SizedBox(height: 12),
        ...items.map((item) => Container(
          margin: const EdgeInsets.only(bottom: 8),
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: Color(themeState.textMain).withValues(alpha: 0.03), 
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
          ),
          child: Row(
            children: [
              Expanded(
                child: Text(item, style: TextStyle(color: Color(themeState.textMain))),
              ),
            ],
          ),
        )),
      ],
    );
  }
}
