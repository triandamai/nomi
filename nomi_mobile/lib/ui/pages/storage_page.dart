import 'dart:ui' show ImageFilter;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/providers/navigation_provider.dart';
import 'package:nomi_mobile/data/models/storage_item.dart';
import 'package:nomi_mobile/ui/widgets/file_preview.dart';

class StoragePage extends ConsumerStatefulWidget {
  const StoragePage({super.key});

  @override
  ConsumerState<StoragePage> createState() => _StoragePageState();
}

class _StoragePageState extends ConsumerState<StoragePage> {
  List<StorageItem> _items = [];
  bool _isLoading = true;
  String _currentPrefix = '';

  @override
  void initState() {
    super.initState();
    _loadStorage();
  }

  Future<void> _loadStorage({String? prefix}) async {
    setState(() => _isLoading = true);
    final items = await ref.read(chatRepositoryProvider).exploreStorage(prefix: prefix);
    if (mounted) {
      setState(() {
        _items = items;
        _currentPrefix = prefix ?? '';
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final themeState = ref.watch(themeProvider);
    final isLargeScreen = MediaQuery.of(context).size.width >= 900;

    return Scaffold(
      backgroundColor: Colors.transparent,
      appBar: isLargeScreen 
        ? null 
        : AppBar(
            backgroundColor: Color(themeState.bgHeader).withValues(alpha: 0.8),
            elevation: 0,
            leading: IconButton(
              onPressed: () => Scaffold.of(context).openDrawer(),
              icon: Icon(LucideIcons.menu, color: Color(themeState.textMain)),
            ),
            title: Text('Storage Explorer', style: TextStyle(color: Color(themeState.textMain), fontSize: 18, fontWeight: FontWeight.bold)),
          ),
      body: Column(
        children: [
          _buildHeader(themeState, isLargeScreen),
          Expanded(
            child: _isLoading 
                ? const Center(child: CircularProgressIndicator())
                : GridView.builder(
                    padding: const EdgeInsets.all(24),
                    gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                      crossAxisCount: 4,
                      crossAxisSpacing: 16,
                      mainAxisSpacing: 16,
                      childAspectRatio: 1.2,
                    ),
                    itemCount: _items.length,
                    itemBuilder: (context, index) => _buildItem(themeState, _items[index]),
                  ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader(NomiTheme themeState, bool isLargeScreen) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(border: Border(bottom: BorderSide(color: Color(themeState.borderMain).withValues(alpha: 0.5)))),
      child: Row(
        children: [
          IconButton(
            onPressed: () => ref.read(navigationProvider.notifier).navigateTo(MainView.chat),
            icon: Icon(LucideIcons.chevronLeft, color: Color(themeState.textMain)),
          ),
          const SizedBox(width: 16),
          if (isLargeScreen) ...[
            Icon(LucideIcons.database, color: Color(themeState.primaryColor), size: 24),
            const SizedBox(width: 16),
          ],
          Text(_currentPrefix.isEmpty ? 'Storage Root' : _currentPrefix, style: TextStyle(color: Color(themeState.textMain), fontSize: 18, fontWeight: FontWeight.bold)),
          const Spacer(),
          if (_currentPrefix.isNotEmpty)
             IconButton(onPressed: () => _loadStorage(prefix: ''), icon: Icon(LucideIcons.home, color: Color(themeState.textMuted))),
        ],
      ),
    );
  }

  Widget _buildItem(NomiTheme themeState, StorageItem item) {
    final isFolder = item.type == 'folder' || item.type == 'bucket';
    return GestureDetector(
      onTap: () {
        if (isFolder) {
          _loadStorage(prefix: item.full_path);
        } else {
          _showFilePreview(themeState, item);
        }
      },
      child: Container(
        decoration: BoxDecoration(
          color: Color(themeState.textMain).withValues(alpha: 0.03),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
        ),
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              isFolder ? LucideIcons.folder : LucideIcons.file, 
              size: 32, 
              color: isFolder ? Colors.amber : Color(themeState.primaryColor)
            ),
            const SizedBox(height: 12),
            Text(
              item.name ?? item.displayPath.split('/').last, 
              textAlign: TextAlign.center,
              style: TextStyle(color: Color(themeState.textMain), fontSize: 12, fontWeight: FontWeight.bold),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ],
        ),
      ),
    );
  }

  void _showFilePreview(NomiTheme themeState, StorageItem item) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (_) => ClipRRect(
        borderRadius: const BorderRadius.only(
          topLeft: Radius.circular(20),
          topRight: Radius.circular(20),
        ),
        child: BackdropFilter(
          filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
          child: Container(
            padding: const EdgeInsets.all(24),
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
            child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(item.name ?? item.displayPath.split('/').last, style: TextStyle(color: Color(themeState.textMain), fontSize: 18, fontWeight: FontWeight.bold)),
            const SizedBox(height: 16),
            SizedBox(
              height: 300,
              child: FilePreviewWidget(
                url: '${AppConfig.baseUrl.replaceFirst('/api', '')}/api/files/${item.path}', 
                mimeType: item.mime_type ?? 'application/octet-stream',
              ),
            ),
            const SizedBox(height: 24),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                _actionButton(LucideIcons.download, 'Download', Color(themeState.primaryColor), () {}),
                _actionButton(LucideIcons.trash2, 'Delete', Colors.red, () {
                  Navigator.pop(context);
                }),
              ],
            ),
          ],
        ),
      ),
      ),
      ),
    );
  }

  Widget _actionButton(IconData icon, String label, Color color, VoidCallback onTap) {
    return Column(
      children: [
        IconButton(onPressed: onTap, icon: Icon(icon, color: color)),
        Text(label, style: TextStyle(color: color, fontSize: 10)),
      ],
    );
  }
}
