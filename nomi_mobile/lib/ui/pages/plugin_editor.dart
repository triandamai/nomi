import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:flutter_highlight/flutter_highlight.dart';
import 'package:flutter_highlight/themes/atom-one-dark.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
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
    final themeState = ref.watch(themeProvider);
    return Scaffold(
      backgroundColor: Colors.transparent,
      body: Column(
        children: [
          _buildHeader(themeState),
          Expanded(
            child: Row(
              children: [
                Expanded(flex: 3, child: _buildMetadataPane(themeState)),
                Expanded(flex: 7, child: _buildEditorPane(themeState)),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader(NomiTheme themeState) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      decoration: BoxDecoration(
        color: Color(themeState.bgHeader),
        border: Border(bottom: BorderSide(color: Color(themeState.borderMain).withValues(alpha: 0.5))),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Row(
            children: [
              IconButton(
                onPressed: () => ref.read(navigationProvider.notifier).navigateTo(MainView.chat),
                icon: Icon(LucideIcons.chevronLeft, color: Color(themeState.textMain)),
              ),
              const SizedBox(width: 12),
              Text(widget.plugin.name, style: TextStyle(color: Color(themeState.textMain), fontSize: 20, fontWeight: FontWeight.w900)),
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

  Widget _buildMetadataPane(NomiTheme themeState) {
    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: Color(themeState.textMain).withValues(alpha: 0.02),
        border: Border(right: BorderSide(color: Color(themeState.borderMain).withValues(alpha: 0.5))),
      ),
      child: ListView(
        children: [
          _buildField(themeState, 'Slug', widget.plugin.slug),
          _buildField(themeState, 'Description', widget.plugin.description ?? 'No description'),
          _buildField(themeState, 'Intents', widget.plugin.intents?.join(', ') ?? 'None'),
          _buildField(themeState, 'Schema', '{ "version": "1.0" }'),
        ],
      ),
    );
  }

  Widget _buildField(NomiTheme themeState, String label, String value) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label.toUpperCase(), style: TextStyle(color: Color(themeState.primaryColor).withValues(alpha: 0.8), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
          const SizedBox(height: 8),
          Text(value, style: TextStyle(color: Color(themeState.textMain), fontSize: 13, fontWeight: FontWeight.bold, height: 1.4)),
        ],
      ),
    );
  }

  Widget _buildEditorPane(NomiTheme themeState) {
    return Container(
      color: themeState.isDark ? const Color(0xFF282c34) : Color(themeState.bgHeader),
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
