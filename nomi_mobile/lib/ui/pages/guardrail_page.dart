import 'dart:ui' show ImageFilter;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/providers/navigation_provider.dart';
import 'package:nomi_mobile/data/models/guardrail_pattern.dart';

class GuardrailPage extends ConsumerStatefulWidget {
  const GuardrailPage({super.key});

  @override
  ConsumerState<GuardrailPage> createState() => _GuardrailPageState();
}

class _GuardrailPageState extends ConsumerState<GuardrailPage> {
  List<GuardrailPattern> _patterns = [];
  bool _isLoading = true;
  final _patternController = TextEditingController();
  bool _isSaving = false;

  @override
  void initState() {
    super.initState();
    _fetchPatterns();
  }

  @override
  void dispose() {
    _patternController.dispose();
    super.dispose();
  }

  Future<void> _fetchPatterns() async {
    if (!mounted) return;
    setState(() => _isLoading = true);
    try {
      final data = await ref.read(chatRepositoryProvider).getGuardrailPatterns();
      if (mounted) {
        setState(() {
          _patterns = data;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  Future<void> _addPattern() async {
    if (_patternController.text.isEmpty) return;
    setState(() => _isSaving = true);
    final success = await ref.read(chatRepositoryProvider).createGuardrailPattern(_patternController.text);
    if (mounted) {
      setState(() => _isSaving = false);
      if (success) {
        _patternController.clear();
        _fetchPatterns();
      } else {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Failed to save pattern.')),
        );
      }
    }
  }

  Future<void> _deletePattern(String id) async {
    final success = await ref.read(chatRepositoryProvider).deleteGuardrailPattern(id);
    if (success) {
      _fetchPatterns();
    } else {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Failed to delete pattern.')),
        );
      }
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
            title: Text('Guardrails', style: TextStyle(color: Color(themeState.textMain), fontSize: 18, fontWeight: FontWeight.bold)),
          ),
      body: Column(
        children: [
          _buildHeader(themeState),
          Expanded(
            child: Row(
              children: [
                Expanded(
                  flex: 6,
                  child: _isLoading 
                    ? const Center(child: CircularProgressIndicator())
                    : _patterns.isEmpty
                      ? Center(child: Text('No guardrail patterns defined.', style: TextStyle(color: Color(themeState.textMuted))))
                      : ListView.builder(
                          padding: const EdgeInsets.all(24),
                          itemCount: _patterns.length,
                          itemBuilder: (context, index) => _PatternItem(
                            pattern: _patterns[index],
                            onDelete: () => _deletePattern(_patterns[index].id),
                          ),
                        ),
                ),
                if (isLargeScreen)
                  Expanded(
                    flex: 4,
                    child: _buildAddPatternPane(themeState),
                  ),
              ],
            ),
          ),
        ],
      ),
      floatingActionButton: isLargeScreen 
        ? null 
        : FloatingActionButton(
            onPressed: () => _showAddPatternSheet(themeState),
            backgroundColor: Color(themeState.primaryColor),
            child: const Icon(LucideIcons.plus, color: Colors.white),
          ),
    );
  }

  Widget _buildHeader(NomiTheme themeState) {
    return Container(
      height: 64,
      padding: const EdgeInsets.symmetric(horizontal: 24),
      decoration: BoxDecoration(
        color: Color(themeState.bgHeader).withValues(alpha: 0.8),
        border: Border(bottom: BorderSide(color: Color(themeState.borderMain).withValues(alpha: 0.5))),
      ),
      child: Row(
        children: [
          IconButton(
            onPressed: () => ref.read(navigationProvider.notifier).navigateTo(MainView.chat),
            icon: Icon(LucideIcons.chevronLeft, color: Color(themeState.textMuted), size: 20),
          ),
          const SizedBox(width: 8),
          Icon(LucideIcons.shieldCheck, color: isDarkTheme(themeState) ? const Color(AppConfig.emerald) : Color(themeState.primaryColor), size: 24),
          const SizedBox(width: 16),
          Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('GUARDRAIL PATTERNS', style: TextStyle(color: Color(themeState.textMain), fontSize: 16, fontWeight: FontWeight.bold)),
              Text('Prompt Injection & Safety Registry', style: TextStyle(color: Color(themeState.textMuted), fontSize: 10, fontWeight: FontWeight.w500)),
            ],
          ),
          const Spacer(),
          IconButton(
            onPressed: _fetchPatterns,
            icon: Icon(LucideIcons.refreshCw, size: 18, color: Color(themeState.textMuted)),
          ),
        ],
      ),
    );
  }

  bool isDarkTheme(NomiTheme themeState) {
    return themeState.isDark;
  }

  Widget _buildAddPatternPane(NomiTheme themeState) {
    return Container(
      padding: const EdgeInsets.all(32),
      decoration: BoxDecoration(
        color: themeState.isDark ? Colors.black26 : Color(themeState.bgHeader).withValues(alpha: 0.3),
        border: Border(left: BorderSide(color: Color(themeState.borderMain).withValues(alpha: 0.5))),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('REGISTER PATTERN', style: TextStyle(color: Color(themeState.primaryColor), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
          const SizedBox(height: 24),
          Text(
            'Add a new pattern to the safety knowledge base. This helps the agent detect and intercept malicious prompt injection attempts.',
            style: TextStyle(color: Color(themeState.textMuted), fontSize: 12, height: 1.5),
          ),
          const SizedBox(height: 32),
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(
              color: Color(themeState.textMain).withValues(alpha: 0.02), 
              borderRadius: BorderRadius.circular(16), 
              border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5))
            ),
            child: TextField(
              controller: _patternController, 
              maxLines: 5, 
              style: TextStyle(color: Color(themeState.textMain), fontSize: 14), 
              decoration: InputDecoration(
                hintText: 'Enter malicious pattern or instruction...', 
                hintStyle: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.5), fontSize: 14), 
                border: InputBorder.none
              )
            ),
          ),
          const SizedBox(height: 32),
          SizedBox(
            width: double.infinity,
            height: 56,
            child: ElevatedButton(
              onPressed: _isSaving ? null : _addPattern,
              style: ElevatedButton.styleFrom(
                backgroundColor: Color(themeState.primaryColor), 
                foregroundColor: Colors.white, 
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)), 
                elevation: 0
              ),
              child: _isSaving 
                ? const SizedBox(width: 24, height: 24, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                : const Text('ENROLL PATTERN', style: TextStyle(fontWeight: FontWeight.w900, letterSpacing: 1)),
            ),
          ),
        ],
      ),
    );
  }

  void _showAddPatternSheet(NomiTheme themeState) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => Padding(
        padding: EdgeInsets.only(bottom: MediaQuery.of(context).viewInsets.bottom),
        child: ClipRRect(
          borderRadius: const BorderRadius.only(
            topLeft: Radius.circular(20),
            topRight: Radius.circular(20),
          ),
          child: BackdropFilter(
            filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
            child: Container(
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
              child: _buildAddPatternPane(themeState)
            ),
          ),
        ),
      ),
    );
  }
}

class _PatternItem extends ConsumerWidget {
  final GuardrailPattern pattern;
  final VoidCallback onDelete;
  const _PatternItem({required this.pattern, required this.onDelete});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        color: Color(themeState.textMain).withValues(alpha: 0.03), 
        borderRadius: BorderRadius.circular(20), 
        border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5))
      ),
      child: Row(
        children: [
          const Icon(LucideIcons.shieldAlert, size: 20, color: Color(AppConfig.rose)),
          const SizedBox(width: 20),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start, 
              children: [
                Text(pattern.content, style: TextStyle(color: Color(themeState.textMain), fontSize: 13, height: 1.4, fontWeight: FontWeight.w500)), 
                const SizedBox(height: 8), 
                Text('ENROLLED: ${pattern.createdAt.split('T')[0]}', style: TextStyle(color: Color(themeState.textMuted), fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1))
              ]
            )
          ),
          IconButton(onPressed: onDelete, icon: Icon(LucideIcons.trash2, size: 16, color: Color(themeState.textMuted).withValues(alpha: 0.5)), hoverColor: const Color(AppConfig.rose).withValues(alpha: 0.1)),
        ],
      ),
    );
  }
}
