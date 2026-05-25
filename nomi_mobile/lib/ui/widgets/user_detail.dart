import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
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
    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: MediaQuery.of(context).size.height * 0.9),
          decoration: BoxDecoration(
            gradient: LinearGradient(
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
              colors: [const Color(AppConfig.deepSlate).withValues(alpha: 0.7), const Color(0xFF1e293b).withValues(alpha: 0.4)],
            ),
            border: const Border(top: BorderSide(color: Colors.white10)),
          ),
          padding: const EdgeInsets.all(24),
          child: _isLoading
              ? const Center(child: CircularProgressIndicator())
              : _user == null
                  ? const Center(child: Text('User not found', style: TextStyle(color: Colors.white)))
                  : SingleChildScrollView(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          _buildHeader(),
                          const SizedBox(height: 24),
                          _buildSection('CONVERSATIONS', _user!.conversations.map((c) => c.title ?? 'Untitled').toList()),
                          const SizedBox(height: 24),
                          _buildSection('CHANNELS', _user!.channels.map((c) => '${c.channelType}: ${c.conversationTitle ?? 'No Title'}').toList()),
                        ],
                      ),
                    ),
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Row(
      children: [
        CircleAvatar(radius: 30, child: Text(_user!.displayName?[0].toUpperCase() ?? '?')),
        const SizedBox(width: 16),
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(_user!.displayName ?? 'Unknown', style: const TextStyle(color: Colors.white, fontSize: 18, fontWeight: FontWeight.bold)),
            Text(_user!.email ?? '', style: const TextStyle(color: Colors.white38)),
          ],
        ),
        const Spacer(),
        IconButton(onPressed: () => Navigator.pop(context), icon: const Icon(LucideIcons.x, color: Colors.white38)),
      ],
    );
  }

  Widget _buildSection(String title, List<String> items) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: const TextStyle(color: Colors.blue, fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
        const SizedBox(height: 12),
        ...items.map((item) => Container(
          margin: const EdgeInsets.only(bottom: 8),
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(color: Colors.white.withValues(alpha: 0.03), borderRadius: BorderRadius.circular(12)),
          child: Text(item, style: const TextStyle(color: Colors.white)),
        )),
      ],
    );
  }
}
