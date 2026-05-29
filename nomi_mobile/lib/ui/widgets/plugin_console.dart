import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/providers/navigation_provider.dart';
import 'package:nomi_mobile/data/models/plugin.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;
import 'dart:ui';

class PluginConsoleSheet extends ConsumerStatefulWidget {
  const PluginConsoleSheet({super.key});

  @override
  ConsumerState<PluginConsoleSheet> createState() => _PluginConsoleSheetState();
}

class _PluginConsoleSheetState extends ConsumerState<PluginConsoleSheet> {
  final _searchController = TextEditingController();

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(chatRepositoryProvider).syncPlugins();
    });
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final themeState = ref.watch(themeProvider);
    final pluginsStream = ref.watch(pluginsStreamProvider(_searchController.text));

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
            border: Border.all(
              color: Color(themeState.borderMain).withValues(alpha: 0.25),
              width: 1.2,
            ),
            borderRadius: const BorderRadius.only(
              topLeft: Radius.circular(20),
              topRight: Radius.circular(20),
            ),
          ),
          child: Column(
        children: [
          // Header with Liquid Glass Feel
          ClipRRect(
            child: BackdropFilter(
              filter: ImageFilter.blur(sigmaX: 10, sigmaY: 10),
              child: Container(
                padding: const EdgeInsets.all(24),
                decoration: BoxDecoration(
                  color: Color(themeState.textMain).withValues(alpha: 0.02),
                  border: Border(bottom: BorderSide(color: Color(themeState.borderMain).withValues(alpha: 0.5))),
                ),
                child: Column(
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'COMPUTE ENGINE',
                              style: TextStyle(
                                color: Color(themeState.accentColor),
                                fontSize: 10,
                                fontWeight: FontWeight.w900,
                                letterSpacing: 2,
                              ),
                            ),
                            const SizedBox(height: 4),
                            Text(
                              'Edge Plugins',
                              style: TextStyle(color: Color(themeState.textMain), fontSize: 22, fontWeight: FontWeight.bold),
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
                    
                    // Search Bar
                    Container(
                      decoration: BoxDecoration(
                        color: themeState.isDark ? Colors.black.withValues(alpha: 0.3) : Color(themeState.textMain).withValues(alpha: 0.05),
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
                      ),
                      child: TextField(
                        controller: _searchController,
                        onChanged: (_) => setState(() {}),
                        style: TextStyle(color: Color(themeState.textMain), fontSize: 14),
                        decoration: InputDecoration(
                          hintText: 'Search plugins or slugs...',
                          hintStyle: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.5), fontSize: 14),
                          prefixIcon: Icon(LucideIcons.search, size: 16, color: Color(themeState.textMuted)),
                          border: InputBorder.none,
                          contentPadding: const EdgeInsets.symmetric(vertical: 14),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),

          // Plugin List
          Expanded(
            child: pluginsStream.when(
              data: (items) {
                if (items.isEmpty) {
                  return Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(LucideIcons.cpu, size: 48, color: Color(themeState.textMuted).withValues(alpha: 0.1)),
                        const SizedBox(height: 16),
                        Text(
                          'No edge plugins available',
                          style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.4), fontSize: 14, fontWeight: FontWeight.bold),
                        ),
                      ],
                    ),
                  );
                }
                return ListView.builder(
                  padding: const EdgeInsets.all(24),
                  itemCount: items.length,
                  itemBuilder: (context, index) {
                    return _PluginListItem(plugin: Plugin.fromDb(items[index]));
                  },
                );
              },
              loading: () => const Center(child: CircularProgressIndicator(strokeWidth: 2)),
              error: (e, _) => Center(child: Text('Sync Error: $e', style: const TextStyle(color: Colors.red))),
            ),
          ),
        ],
      ),
    ),
    ),
    );
  }
}

class _PluginListItem extends ConsumerWidget {
  final Plugin plugin;
  const _PluginListItem({required this.plugin});

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
          Navigator.pop(context); // Close sheet
          ref.read(navigationProvider.notifier).navigateTo(
            MainView.pluginEditor,
            args: {'plugin': plugin},
          );
        },
        borderRadius: BorderRadius.circular(20),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Color(themeState.primaryColor).withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(16),
                ),
                child: Icon(LucideIcons.code2, size: 20, color: Color(themeState.primaryColor)),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Text(
                          plugin.name,
                          style: TextStyle(color: Color(themeState.textMain), fontSize: 14, fontWeight: FontWeight.bold),
                        ),
                        const SizedBox(width: 8),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: Color(themeState.textMain).withValues(alpha: 0.05),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            'V${plugin.version ?? "1.0"}',
                            style: TextStyle(color: Color(themeState.textMuted), fontSize: 7, fontWeight: FontWeight.w900, letterSpacing: 1),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 4),
                    Text(
                      plugin.slug,
                      style: TextStyle(color: Color(themeState.primaryColor), fontSize: 10, fontFamily: 'monospace'),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      plugin.description ?? 'No description provided.',
                      style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.8), fontSize: 12),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ),
              ),
              Icon(LucideIcons.chevronRight, size: 16, color: Color(themeState.textMuted).withValues(alpha: 0.3)),
            ],
          ),
        ),
      ),
    );
  }
}

final pluginsStreamProvider = StreamProvider.family<List<db.Plugin>, String>((ref, search) {
  return ref.watch(chatRepositoryProvider).watchPlugins(search: search);
});
