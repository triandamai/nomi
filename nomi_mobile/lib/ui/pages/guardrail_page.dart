import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
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
    final isLargeScreen = MediaQuery.of(context).size.width >= 900;

    return Scaffold(
      backgroundColor: Colors.transparent,
      appBar: isLargeScreen 
        ? null 
        : AppBar(
            backgroundColor: const Color(AppConfig.deepSlate).withValues(alpha: 0.8),
            elevation: 0,
            leading: IconButton(
              onPressed: () => Scaffold.of(context).openDrawer(),
              icon: const Icon(LucideIcons.menu),
            ),
            title: const Text('Guardrails', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
          ),
      body: Column(
        children: [
          _buildHeader(),
          Expanded(
            child: Row(
              children: [
                Expanded(
                  flex: 6,
                  child: _isLoading 
                    ? const Center(child: CircularProgressIndicator())
                    : _patterns.isEmpty
                      ? const Center(child: Text('No guardrail patterns defined.', style: TextStyle(color: Colors.white38)))
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
                    child: _buildAddPatternPane(),
                  ),
              ],
            ),
          ),
        ],
      ),
      floatingActionButton: isLargeScreen 
        ? null 
        : FloatingActionButton(
            onPressed: () => _showAddPatternSheet(),
            backgroundColor: const Color(AppConfig.blue),
            child: const Icon(LucideIcons.plus, color: Colors.white),
          ),
    );
  }

  Widget _buildHeader() {
    return Container(
      height: 64,
      padding: const EdgeInsets.symmetric(horizontal: 24),
      decoration: BoxDecoration(
        color: const Color(AppConfig.deepSlate).withValues(alpha: 0.8),
        border: Border(bottom: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
      ),
      child: Row(
        children: [
          IconButton(
            onPressed: () => ref.read(navigationProvider.notifier).navigateTo(MainView.chat),
            icon: const Icon(LucideIcons.chevronLeft, color: Colors.white38, size: 20),
          ),
          const SizedBox(width: 8),
          const Icon(LucideIcons.shieldCheck, color: Color(AppConfig.emerald), size: 24),
          const SizedBox(width: 16),
          const Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('GUARDRAIL PATTERNS', style: TextStyle(color: Colors.white, fontSize: 16, fontWeight: FontWeight.bold)),
              Text('Prompt Injection & Safety Registry', style: TextStyle(color: Colors.white38, fontSize: 10, fontWeight: FontWeight.w500)),
            ],
          ),
          const Spacer(),
          IconButton(
            onPressed: _fetchPatterns,
            icon: const Icon(LucideIcons.refreshCw, size: 18, color: Colors.white38),
          ),
        ],
      ),
    );
  }

  Widget _buildAddPatternPane() {
    return Container(
      padding: const EdgeInsets.all(32),
      decoration: const BoxDecoration(
        color: Colors.black26,
        border: Border(left: BorderSide(color: Colors.white10)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('REGISTER PATTERN', style: TextStyle(color: Colors.blue, fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
          const SizedBox(height: 24),
          const Text(
            'Add a new pattern to the safety knowledge base. This helps the agent detect and intercept malicious prompt injection attempts.',
            style: TextStyle(color: Colors.white38, fontSize: 12, height: 1.5),
          ),
          const SizedBox(height: 32),
          Container(
            padding: const EdgeInsets.all(20),
            decoration: BoxDecoration(color: Colors.white.withValues(alpha: 0.02), borderRadius: BorderRadius.circular(16), border: Border.all(color: Colors.white.withValues(alpha: 0.05))),
            child: TextField(controller: _patternController, maxLines: 5, style: const TextStyle(color: Colors.white, fontSize: 14), decoration: const InputDecoration(hintText: 'Enter malicious pattern or instruction...', hintStyle: TextStyle(color: Colors.white24, fontSize: 14), border: InputBorder.none)),
          ),
          const SizedBox(height: 32),
          SizedBox(
            width: double.infinity,
            height: 56,
            child: ElevatedButton(
              onPressed: _isSaving ? null : _addPattern,
              style: ElevatedButton.styleFrom(backgroundColor: const Color(AppConfig.blue), foregroundColor: Colors.white, shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)), elevation: 0),
              child: _isSaving 
                ? const SizedBox(width: 24, height: 24, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                : const Text('ENROLL PATTERN', style: TextStyle(fontWeight: FontWeight.w900, letterSpacing: 1)),
            ),
          ),
        ],
      ),
    );
  }

  void _showAddPatternSheet() {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => Padding(
        padding: EdgeInsets.only(bottom: MediaQuery.of(context).viewInsets.bottom),
        child: Container(decoration: const BoxDecoration(color: Color(AppConfig.deepSlate), borderRadius: BorderRadius.vertical(top: Radius.circular(24))), child: _buildAddPatternPane()),
      ),
    );
  }
}

class _PatternItem extends StatelessWidget {
  final GuardrailPattern pattern;
  final VoidCallback onDelete;
  const _PatternItem({required this.pattern, required this.onDelete});

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(color: Colors.white.withValues(alpha: 0.03), borderRadius: BorderRadius.circular(20), border: Border.all(color: Colors.white.withValues(alpha: 0.05))),
      child: Row(
        children: [
          const Icon(LucideIcons.shieldAlert, size: 20, color: Color(AppConfig.rose)),
          const SizedBox(width: 20),
          Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [Text(pattern.content, style: const TextStyle(color: Colors.white, fontSize: 13, height: 1.4, fontWeight: FontWeight.w500)), const SizedBox(height: 8), Text('ENROLLED: ${pattern.createdAt.split('T')[0]}', style: const TextStyle(color: Colors.white24, fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1))])),
          IconButton(onPressed: onDelete, icon: const Icon(LucideIcons.trash2, size: 16, color: Colors.white12), hoverColor: Color(AppConfig.rose).withValues(alpha: 0.1)),
        ],
      ),
    );
  }
}
