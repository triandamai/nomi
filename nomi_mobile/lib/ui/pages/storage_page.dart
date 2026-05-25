import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/data/models/storage_item.dart';
import 'package:nomi_mobile/ui/widgets/sidebar.dart';
import 'package:nomi_mobile/ui/widgets/file_preview.dart';
import 'package:path/path.dart' as p;

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
    final isLargeScreen = MediaQuery.of(context).size.width >= 900;

    return Scaffold(
      backgroundColor: const Color(AppConfig.deepSlate),
      body: Row(
        children: [
          if (isLargeScreen) const NomiSidebar(isDrawer: false),
          Expanded(
            child: Column(
              children: [
                _buildHeader(),
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
                          itemBuilder: (context, index) => _buildItem(_items[index]),
                        ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader() {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(border: Border(bottom: BorderSide(color: Colors.white.withValues(alpha: 0.05)))),
      child: Row(
        children: [
          IconButton(onPressed: () => Navigator.pop(context), icon: const Icon(LucideIcons.chevronLeft, color: Colors.white)),
          const SizedBox(width: 16),
          Text(_currentPrefix.isEmpty ? 'Storage Root' : _currentPrefix, style: const TextStyle(color: Colors.white, fontSize: 18, fontWeight: FontWeight.bold)),
          const Spacer(),
          if (_currentPrefix.isNotEmpty)
             IconButton(onPressed: () => _loadStorage(prefix: ''), icon: const Icon(LucideIcons.home, color: Colors.white38)),
        ],
      ),
    );
  }

  Widget _buildItem(StorageItem item) {
    final isFolder = item.type == 'folder' || item.type == 'bucket';
    return GestureDetector(
      onTap: () {
        if (isFolder) {
          _loadStorage(prefix: item.full_path);
        } else {
          _showFilePreview(item);
        }
      },
      child: Container(
        decoration: BoxDecoration(
          color: Colors.white.withValues(alpha: 0.03),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
        ),
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(isFolder ? LucideIcons.folder : LucideIcons.file, size: 32, color: isFolder ? Colors.amber : Colors.blue),
            const SizedBox(height: 12),
            Text(
              item.name ?? item.displayPath.split('/').last, 
              textAlign: TextAlign.center,
              style: const TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ],
        ),
      ),
    );
  }

  void _showFilePreview(StorageItem item) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (_) => Container(
        padding: const EdgeInsets.all(24),
        decoration: const BoxDecoration(color: Color(AppConfig.deepSlate), borderRadius: BorderRadius.vertical(top: Radius.circular(24))),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(item.name ?? item.displayPath.split('/').last, style: const TextStyle(color: Colors.white, fontSize: 18, fontWeight: FontWeight.bold)),
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
                _actionButton(LucideIcons.download, 'Download', Colors.blue, () {}),
                _actionButton(LucideIcons.trash2, 'Delete', Colors.red, () {
                  Navigator.pop(context);
                }),
              ],
            ),
          ],
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
