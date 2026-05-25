import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/data/models/user_profile.dart';
import 'package:nomi_mobile/ui/widgets/user_detail.dart';

class UserDirectorySheet extends ConsumerStatefulWidget {
  const UserDirectorySheet({super.key});

  @override
  ConsumerState<UserDirectorySheet> createState() => _UserDirectorySheetState();
}

class _UserDirectorySheetState extends ConsumerState<UserDirectorySheet> {
  List<UserProfile> _users = [];
  bool _isLoading = true;
  final _searchController = TextEditingController();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _fetchUsers());
  }

  Future<void> _fetchUsers({String? query}) async {
    setState(() => _isLoading = true);
    try {
      final data = await ref.read(chatRepositoryProvider).getUsers(query: query);
      if (mounted) {
        setState(() {
          _users = data;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final size = MediaQuery.of(context).size;
    final isLargeScreen = size.width >= 700;

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
            border: const Border(top: BorderSide(color: Colors.white10)),
          ),
          padding: EdgeInsets.symmetric(horizontal: 24, vertical: isLargeScreen ? 24 : 32),
          child: SafeArea(
            child: Column(
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('DIRECTORY', style: TextStyle(color: Color(AppConfig.blue), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
                        SizedBox(height: 4),
                        Text('User Registry', style: TextStyle(color: Colors.white, fontSize: 22, fontWeight: FontWeight.bold)),
                      ],
                    ),
                    IconButton(
                      onPressed: () => Navigator.pop(context),
                      icon: const Icon(LucideIcons.x, color: Colors.white38),
                    ),
                  ],
                ),
                const SizedBox(height: 24),

                Container(
                  decoration: BoxDecoration(
                    color: Colors.black.withValues(alpha: 0.3),
                    borderRadius: BorderRadius.circular(16),
                    border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
                  ),
                  child: TextField(
                    controller: _searchController,
                    onSubmitted: (val) => _fetchUsers(query: val),
                    style: const TextStyle(color: Colors.white, fontSize: 14),
                    decoration: const InputDecoration(
                      hintText: 'Search by name or email...',
                      hintStyle: TextStyle(color: Colors.white24, fontSize: 14),
                      prefixIcon: Icon(LucideIcons.search, size: 16, color: Colors.white24),
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.symmetric(vertical: 14),
                    ),
                  ),
                ),
                const SizedBox(height: 24),
                
                Expanded(
                  child: _isLoading 
                    ? const Center(child: CircularProgressIndicator())
                    : ListView.builder(
                        itemCount: _users.length,
                        itemBuilder: (context, index) => _UserItem(user: _users[index]),
                      ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _UserItem extends StatelessWidget {
  final UserProfile user;
  const _UserItem({required this.user});

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.03),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
      ),
      child: InkWell(
        onTap: () {
          showModalBottomSheet(
            context: context,
            isScrollControlled: true,
            backgroundColor: Colors.transparent,
            builder: (context) => UserDetailSheet(userId: user.id),
          );
        },
        borderRadius: BorderRadius.circular(20),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              CircleAvatar(
                backgroundColor: const Color(AppConfig.blue).withValues(alpha: 0.1),
                child: Text(user.displayName?[0].toUpperCase() ?? '?', style: const TextStyle(color: Color(AppConfig.blue))),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(user.displayName ?? user.name ?? 'Unknown', style: const TextStyle(color: Colors.white, fontSize: 14, fontWeight: FontWeight.bold)),
                    Text(user.email ?? 'No email', style: const TextStyle(color: Colors.white38, fontSize: 11, fontFamily: 'monospace')),
                  ],
                ),
              ),
              if (user.isVerified ?? false)
                const Icon(LucideIcons.checkCircle2, color: Color(AppConfig.emerald), size: 16),
            ],
          ),
        ),
      ),
    );
  }
}
