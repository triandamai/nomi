import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
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
    final themeState = ref.watch(themeProvider);
    final size = MediaQuery.of(context).size;
    final isLargeScreen = size.width >= 700;

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
            border: Border.all(
              color: Color(themeState.borderMain).withValues(alpha: 0.25),
              width: 1.2,
            ),
            borderRadius: const BorderRadius.only(
              topLeft: Radius.circular(20),
              topRight: Radius.circular(20),
            ),
          ),
          padding: EdgeInsets.symmetric(horizontal: 24, vertical: isLargeScreen ? 24 : 32),
          child: SafeArea(
            child: Column(
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'DIRECTORY', 
                          style: TextStyle(
                            color: Color(themeState.primaryColor), 
                            fontSize: 10, 
                            fontWeight: FontWeight.w900, 
                            letterSpacing: 2
                          )
                        ),
                        const SizedBox(height: 4),
                        Text(
                          'User Registry', 
                          style: TextStyle(color: Color(themeState.textMain), fontSize: 22, fontWeight: FontWeight.bold)
                        ),
                      ],
                    ),
                    IconButton(
                      onPressed: () => Navigator.pop(context),
                      icon: Icon(LucideIcons.x, color: Color(themeState.textMuted)),
                    ),
                  ],
                ),
                const SizedBox(height: 24),

                Container(
                  decoration: BoxDecoration(
                    color: themeState.isDark ? Colors.black.withValues(alpha: 0.3) : Color(themeState.textMain).withValues(alpha: 0.05),
                    borderRadius: BorderRadius.circular(16),
                    border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
                  ),
                  child: TextField(
                    controller: _searchController,
                    onSubmitted: (val) => _fetchUsers(query: val),
                    style: TextStyle(color: Color(themeState.textMain), fontSize: 14),
                    decoration: InputDecoration(
                      hintText: 'Search by name or email...',
                      hintStyle: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.5), fontSize: 14),
                      prefixIcon: Icon(LucideIcons.search, size: 16, color: Color(themeState.textMuted)),
                      border: InputBorder.none,
                      contentPadding: const EdgeInsets.symmetric(vertical: 14),
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

class _UserItem extends ConsumerWidget {
  final UserProfile user;
  const _UserItem({required this.user});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      decoration: BoxDecoration(
        color: Color(themeState.textMain).withValues(alpha: 0.03),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
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
                backgroundColor: Color(themeState.primaryColor).withValues(alpha: 0.1),
                child: Text(
                  user.displayName?[0].toUpperCase() ?? '?', 
                  style: TextStyle(color: Color(themeState.primaryColor))
                ),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      user.displayName ?? user.name ?? 'Unknown', 
                      style: TextStyle(color: Color(themeState.textMain), fontSize: 14, fontWeight: FontWeight.bold)
                    ),
                    Text(
                      user.email ?? 'No email', 
                      style: TextStyle(color: Color(themeState.textMuted), fontSize: 11, fontFamily: 'monospace')
                    ),
                  ],
                ),
              ),
              if (user.isVerified ?? false)
                Icon(LucideIcons.checkCircle2, color: Color(themeState.accentColor), size: 16),
            ],
          ),
        ),
      ),
    );
  }
}
