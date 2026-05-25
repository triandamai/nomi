import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/providers/navigation_provider.dart';
import 'package:nomi_mobile/data/models/skill.dart';
import 'package:flutter_highlight/flutter_highlight.dart';
import 'package:flutter_highlight/themes/atom-one-dark.dart';
import 'dart:convert';

class SkillsPage extends ConsumerStatefulWidget {
  const SkillsPage({super.key});

  @override
  ConsumerState<SkillsPage> createState() => _SkillsPageState();
}

class _SkillsPageState extends ConsumerState<SkillsPage> {
  List<Skill> _skills = [];
  bool _isLoading = true;
  String _searchQuery = '';

  @override
  void initState() {
    super.initState();
    _fetchSkills();
  }

  Future<void> _fetchSkills() async {
    if (!mounted) return;
    setState(() => _isLoading = true);
    try {
      final data = await ref.read(chatRepositoryProvider).getSkills();
      if (mounted) {
        setState(() {
          _skills = data;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  List<Skill> get _filteredSkills {
    if (_searchQuery.isEmpty) return _skills;
    return _skills.where((s) => 
      s.name.toLowerCase().contains(_searchQuery.toLowerCase()) || 
      s.description.toLowerCase().contains(_searchQuery.toLowerCase()) ||
      s.intents.any((i) => i.toLowerCase().contains(_searchQuery.toLowerCase()))
    ).toList();
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
            title: const Text('System Skills', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
          ),
      body: Column(
        children: [
          _buildHeader(),
          _buildSearchBar(),
          Expanded(
            child: _isLoading 
              ? const Center(child: CircularProgressIndicator())
              : _filteredSkills.isEmpty
                ? const Center(child: Text('No matching skills found.', style: TextStyle(color: Colors.white38)))
                : GridView.builder(
                    padding: const EdgeInsets.all(32),
                    gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                      crossAxisCount: isLargeScreen ? 3 : 2,
                      crossAxisSpacing: 24,
                      mainAxisSpacing: 24,
                      childAspectRatio: 1.3,
                    ),
                    itemCount: _filteredSkills.length,
                    itemBuilder: (context, index) => _SkillCard(
                      skill: _filteredSkills[index],
                      onTap: () => _showSkillDetail(_filteredSkills[index]),
                    ),
                  ),
          ),
        ],
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
          const Icon(LucideIcons.puzzle, color: Color(AppConfig.indigo), size: 24),
          const SizedBox(width: 16),
          const Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('SYSTEM SKILLS', style: TextStyle(color: Colors.white, fontSize: 16, fontWeight: FontWeight.bold)),
              Text('Registry of Native & Dynamic Capabilities', style: TextStyle(color: Colors.white38, fontSize: 10, fontWeight: FontWeight.w500)),
            ],
          ),
          const Spacer(),
          IconButton(
            onPressed: _fetchSkills,
            icon: const Icon(LucideIcons.refreshCw, size: 18, color: Colors.white38),
          ),
        ],
      ),
    );
  }

  Widget _buildSearchBar() {
    return Container(
      padding: const EdgeInsets.fromLTRB(32, 24, 32, 0),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 4),
        decoration: BoxDecoration(
          color: Colors.white.withValues(alpha: 0.02),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: Colors.white10),
        ),
        child: TextField(
          onChanged: (val) => setState(() => _searchQuery = val),
          style: const TextStyle(color: Colors.white, fontSize: 14),
          decoration: const InputDecoration(
            hintText: 'Search skills, descriptions, or intents...',
            hintStyle: TextStyle(color: Colors.white24, fontSize: 14),
            prefixIcon: Icon(LucideIcons.search, size: 16, color: Colors.white24),
            border: InputBorder.none,
          ),
        ),
      ),
    );
  }

  void _showSkillDetail(Skill skill) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => _SkillDetailSheet(skill: skill),
    );
  }
}

class _SkillCard extends StatelessWidget {
  final Skill skill;
  final VoidCallback onTap;

  const _SkillCard({required this.skill, required this.onTap});

  IconData _getIcon(String name) {
    final n = name.toLowerCase();
    if (n.contains('finance') || n.contains('money')) return LucideIcons.creditCard;
    if (n.contains('health') || n.contains('vitality')) return LucideIcons.activity;
    if (n.contains('media') || n.contains('vision')) return LucideIcons.image;
    if (n.contains('audio') || n.contains('voice')) return LucideIcons.mic;
    if (n.contains('web') || n.contains('search')) return LucideIcons.globe;
    if (n.contains('remind') || n.contains('schedule') || n.contains('task')) return LucideIcons.bell;
    if (n.contains('doc') || n.contains('file') || n.contains('knowledge')) return LucideIcons.fileText;
    return LucideIcons.wrench;
  }

  @override
  Widget build(BuildContext context) {
    final isSystem = skill.skillType == 'System';
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: onTap,
        child: Container(
          decoration: BoxDecoration(
            color: Colors.white.withValues(alpha: 0.02),
            borderRadius: BorderRadius.circular(20),
            border: Border.all(color: Colors.white10),
          ),
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Container(
                    padding: const EdgeInsets.all(10),
                    decoration: BoxDecoration(
                      color: Colors.black.withValues(alpha: 0.3),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: Icon(_getIcon(skill.name), color: isSystem ? const Color(AppConfig.emerald) : const Color(AppConfig.indigo), size: 20),
                  ),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: (isSystem ? const Color(AppConfig.emerald) : const Color(AppConfig.indigo)).withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Text(
                      skill.skillType.toUpperCase(),
                      style: TextStyle(color: isSystem ? const Color(AppConfig.emerald) : const Color(AppConfig.indigo), fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1),
                    ),
                  ),
                ],
              ),
              const Spacer(),
              Text(skill.name, style: const TextStyle(color: Colors.white, fontSize: 14, fontWeight: FontWeight.bold)),
              const SizedBox(height: 8),
              Text(
                skill.description,
                style: TextStyle(color: Colors.white.withValues(alpha: 0.3), fontSize: 11, height: 1.4),
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _SkillDetailSheet extends StatefulWidget {
  final Skill skill;
  const _SkillDetailSheet({required this.skill});

  @override
  State<_SkillDetailSheet> createState() => _SkillDetailSheetState();
}

class _SkillDetailSheetState extends State<_SkillDetailSheet> {
  bool _showCode = false;
  bool _showSchema = false;

  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: MediaQuery.of(context).size.height * 0.85),
          decoration: BoxDecoration(
            color: const Color(AppConfig.deepSlate).withValues(alpha: 0.9),
            borderRadius: const BorderRadius.vertical(top: Radius.circular(24)),
            border: const Border(top: BorderSide(color: Colors.white10)),
          ),
          padding: const EdgeInsets.all(32),
          child: SingleChildScrollView(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _buildHeader(),
                const SizedBox(height: 32),
                _section('DESCRIPTION', widget.skill.description),
                const SizedBox(height: 24),
                _buildIntents(),
                if (widget.skill.scriptCode != null) ...[
                  const SizedBox(height: 24),
                  _buildToggleSection('SOURCE CODE', _showCode, (val) => setState(() => _showCode = val), widget.skill.scriptCode!, isCode: true),
                ],
                if (widget.skill.schemaJson != null) ...[
                  const SizedBox(height: 24),
                  _buildToggleSection('TECHNICAL SCHEMA', _showSchema, (val) => setState(() => _showSchema = val), jsonEncode(widget.skill.schemaJson)),
                ],
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
        const Icon(LucideIcons.sparkles, color: Color(AppConfig.indigo), size: 24),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('SKILL INTELLIGENCE', style: TextStyle(color: Color(AppConfig.indigo), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
              Text(widget.skill.name, style: const TextStyle(color: Colors.white, fontSize: 20, fontWeight: FontWeight.bold)),
            ],
          ),
        ),
        IconButton(onPressed: () => Navigator.pop(context), icon: const Icon(LucideIcons.x, color: Colors.white38)),
      ],
    );
  }

  Widget _section(String title, String content) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: const TextStyle(color: Colors.white24, fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
        const SizedBox(height: 12),
        Text(content, style: const TextStyle(color: Colors.white70, fontSize: 14, height: 1.6)),
      ],
    );
  }

  Widget _buildIntents() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('ACTIVATION INTENTS', style: TextStyle(color: Colors.white24, fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: widget.skill.intents.map((i) => Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            decoration: BoxDecoration(
              color: Colors.white.withValues(alpha: 0.05),
              borderRadius: BorderRadius.circular(6),
            ),
            child: Text(i, style: const TextStyle(color: Colors.white70, fontSize: 11, fontFamily: 'monospace')),
          )).toList(),
        ),
      ],
    );
  }

  Widget _buildToggleSection(String title, bool isExpanded, Function(bool) onToggle, String content, {bool isCode = false}) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        GestureDetector(
          onTap: () => onToggle(!isExpanded),
          child: Row(
            children: [
              Text(title, style: const TextStyle(color: Colors.white24, fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
              const Spacer(),
              Icon(isExpanded ? LucideIcons.chevronUp : LucideIcons.chevronDown, size: 14, color: Colors.white24),
            ],
          ),
        ),
        if (isExpanded)
          Container(
            margin: const EdgeInsets.only(top: 12),
            width: double.infinity,
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Colors.black,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: Colors.white10),
            ),
            child: isCode 
              ? HighlightView(
                  content,
                  language: 'typescript',
                  theme: atomOneDarkTheme,
                  padding: const EdgeInsets.all(12),
                  textStyle: const TextStyle(fontFamily: 'monospace', fontSize: 12),
                )
              : Text(content, style: const TextStyle(color: Color(AppConfig.emerald), fontSize: 11, fontFamily: 'monospace')),
          ),
      ],
    );
  }
}
