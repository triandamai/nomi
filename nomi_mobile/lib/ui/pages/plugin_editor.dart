import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:flutter_highlight/flutter_highlight.dart';
import 'package:flutter_highlight/themes/atom-one-dark.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/data/models/plugin.dart';
import 'package:nomi_mobile/providers/navigation_provider.dart';

class PluginEditorPage extends ConsumerStatefulWidget {
  final Plugin plugin;

  const PluginEditorPage({super.key, required this.plugin});

  @override
  ConsumerState<PluginEditorPage> createState() => _PluginEditorPageState();
}

class _PluginEditorPageState extends ConsumerState<PluginEditorPage> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.transparent,
      body: Column(
        children: [
          _buildHeader(),
          Expanded(
            child: Row(
              children: [
                Expanded(flex: 3, child: _buildMetadataPane()),
                Expanded(flex: 7, child: _buildEditorPane()),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      decoration: BoxDecoration(
        color: const Color(AppConfig.deepSlate),
        border: Border(bottom: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Row(
            children: [
              IconButton(
                onPressed: () => ref.read(navigationProvider.notifier).navigateTo(MainView.chat),
                icon: const Icon(LucideIcons.chevronLeft, color: Colors.white),
              ),
              const SizedBox(width: 12),
              Text(widget.plugin.name, style: const TextStyle(color: Colors.white, fontSize: 20, fontWeight: FontWeight.w900)),
            ],
          ),
          Row(
            children: [
              _ActionButton(icon: LucideIcons.play, label: 'Run Test', color: const Color(AppConfig.emerald), onTap: () {}),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildMetadataPane() {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.02),
        border: const Border(right: BorderSide(color: Colors.white10)),
      ),
      child: ListView(
        children: [
          _buildField('Slug', widget.plugin.slug),
          _buildField('Description', widget.plugin.description ?? 'No description'),
          _buildField('Intents', widget.plugin.intents?.join(', ') ?? 'None'),
          _buildField('Schema', '{ "version": "1.0" }'),
        ],
      ),
    );
  }

  Widget _buildField(String label, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label.toUpperCase(), style: TextStyle(color: Colors.blue.withValues(alpha: 0.5), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
          const SizedBox(height: 8),
          Text(value, style: const TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.bold, height: 1.4)),
        ],
      ),
    );
  }

  Widget _buildEditorPane() {
    return Container(
      color: const Color(0xFF282c34),
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: HighlightView(
          widget.plugin.scriptCode ?? '// No code available.',
          language: 'typescript',
          theme: atomOneDarkTheme,
          padding: const EdgeInsets.all(12),
          textStyle: const TextStyle(fontFamily: 'monospace', fontSize: 13),
        ),
      ),
    );
  }
}

class _ActionButton extends StatelessWidget {
  final IconData icon;
  final String label;
  final Color color;
  final VoidCallback onTap;

  const _ActionButton({required this.icon, required this.label, required this.color, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return TextButton.icon(
      onPressed: onTap,
      icon: Icon(icon, size: 14, color: color),
      label: Text(label.toUpperCase(), style: TextStyle(color: color, fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 1)),
      style: TextButton.styleFrom(
        backgroundColor: color.withValues(alpha: 0.1),
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      ),
    );
  }
}
