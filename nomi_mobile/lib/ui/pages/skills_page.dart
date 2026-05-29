import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
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
            title: Text('System Skills', style: TextStyle(color: Color(themeState.textMain), fontSize: 18, fontWeight: FontWeight.bold)),
          ),
      body: CustomScrollView(
        slivers: [
          SliverToBoxAdapter(child: _buildHeader(themeState)),
          SliverToBoxAdapter(child: _buildSearchBar(themeState)),
          _isLoading 
            ? const SliverFillRemaining(child: Center(child: CircularProgressIndicator()))
            : _filteredSkills.isEmpty
              ? SliverFillRemaining(child: Center(child: Text('No matching skills found.', style: TextStyle(color: Color(themeState.textMuted)))))
              : SliverPadding(
                  padding: EdgeInsets.all(isLargeScreen ? 32 : 16),
                  sliver: isLargeScreen 
                    ? SliverGrid(
                        gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                          crossAxisCount: 3,
                          crossAxisSpacing: 24,
                          mainAxisSpacing: 24,
                          childAspectRatio: 1.3,
                        ),
                        delegate: SliverChildBuilderDelegate(
                          (context, index) => _SkillCard(
                            skill: _filteredSkills[index],
                            onTap: () => _showSkillDetail(_filteredSkills[index]),
                          ),
                          childCount: _filteredSkills.length,
                        ),
                      )
                    : SliverList(
                        delegate: SliverChildBuilderDelegate(
                          (context, index) => Padding(
                            padding: const EdgeInsets.only(bottom: 12),
                            child: _SkillCard(
                              skill: _filteredSkills[index],
                              onTap: () => _showSkillDetail(_filteredSkills[index]),
                            ),
                          ),
                          childCount: _filteredSkills.length,
                        ),
                      ),
                ),
        ],
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
          Icon(LucideIcons.puzzle, color: Color(themeState.primaryColor), size: 24),
          const SizedBox(width: 16),
          Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('SYSTEM SKILLS', style: TextStyle(color: Color(themeState.textMain), fontSize: 16, fontWeight: FontWeight.bold)),
              Text('Registry of Native & Dynamic Capabilities', style: TextStyle(color: Color(themeState.textMuted), fontSize: 10, fontWeight: FontWeight.w500)),
            ],
          ),
          const Spacer(),
          IconButton(
            onPressed: _fetchSkills,
            icon: Icon(LucideIcons.refreshCw, size: 18, color: Color(themeState.textMuted)),
          ),
        ],
      ),
    );
  }

  Widget _buildSearchBar(NomiTheme themeState) {
    return Container(
      padding: const EdgeInsets.fromLTRB(32, 24, 32, 0),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 4),
        decoration: BoxDecoration(
          color: Color(themeState.textMain).withValues(alpha: 0.02),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
        ),
        child: TextField(
          onChanged: (val) => setState(() => _searchQuery = val),
          style: TextStyle(color: Color(themeState.textMain), fontSize: 14),
          decoration: InputDecoration(
            hintText: 'Search skills, descriptions, or intents...',
            hintStyle: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.5), fontSize: 14),
            prefixIcon: Icon(LucideIcons.search, size: 16, color: Color(themeState.textMuted)),
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

class _SkillCard extends ConsumerWidget {
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
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    final isLargeScreen = MediaQuery.of(context).size.width >= 900;
    final isSystem = skill.skillType == 'System';

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: onTap,
        child: Container(
          height: isLargeScreen ? null : 160,
          decoration: BoxDecoration(
            color: Color(themeState.textMain).withValues(alpha: 0.02),
            borderRadius: BorderRadius.circular(20),
            border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
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
                      color: themeState.isDark ? Colors.black.withValues(alpha: 0.3) : Color(themeState.textMain).withValues(alpha: 0.05),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: Icon(_getIcon(skill.name), color: isSystem ? const Color(AppConfig.emerald) : Color(themeState.primaryColor), size: 20),
                  ),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: (isSystem ? const Color(AppConfig.emerald) : Color(themeState.primaryColor)).withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Text(
                      skill.skillType.toUpperCase(),
                      style: TextStyle(color: isSystem ? const Color(AppConfig.emerald) : Color(themeState.primaryColor), fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1),
                    ),
                  ),
                ],
              ),
              const Spacer(),
              Text(skill.name, style: TextStyle(color: Color(themeState.textMain), fontSize: 14, fontWeight: FontWeight.bold)),
              const SizedBox(height: 8),
              Flexible(
                child: Text(
                  skill.description,
                  style: TextStyle(color: Color(themeState.textMuted), fontSize: 11, height: 1.4),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _SkillDetailSheet extends ConsumerStatefulWidget {
  final Skill skill;
  const _SkillDetailSheet({required this.skill});

  @override
  ConsumerState<_SkillDetailSheet> createState() => _SkillDetailSheetState();
}

class _SkillDetailSheetState extends ConsumerState<_SkillDetailSheet> {
  bool _showCode = false;
  bool _showSchema = false;

  @override
  Widget build(BuildContext context) {
    final themeState = ref.watch(themeProvider);
    return ClipRRect(
      borderRadius: const BorderRadius.only(
        topLeft: Radius.circular(20),
        topRight: Radius.circular(20),
      ),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: MediaQuery.of(context).size.height * 0.85),
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
          padding: const EdgeInsets.all(32),
          child: SingleChildScrollView(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _buildHeader(themeState),
                const SizedBox(height: 32),
                _section(themeState, 'DESCRIPTION', widget.skill.description),
                const SizedBox(height: 24),
                _buildIntents(themeState),
                if (widget.skill.scriptCode != null) ...[
                  const SizedBox(height: 24),
                  _buildToggleSection(themeState, 'SOURCE CODE', _showCode, (val) => setState(() => _showCode = val), widget.skill.scriptCode!, isCode: true),
                ],
                if (widget.skill.schemaJson != null) ...[
                  const SizedBox(height: 24),
                  _buildToggleSection(themeState, 'TECHNICAL SCHEMA', _showSchema, (val) => setState(() => _showSchema = val), jsonEncode(widget.skill.schemaJson)),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildHeader(NomiTheme themeState) {
    return Row(
      children: [
        Icon(LucideIcons.sparkles, color: Color(themeState.primaryColor), size: 24),
        const SizedBox(width: 16),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('SKILL INTELLIGENCE', style: TextStyle(color: Color(themeState.primaryColor), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
              Text(widget.skill.name, style: TextStyle(color: Color(themeState.textMain), fontSize: 20, fontWeight: FontWeight.bold)),
            ],
          ),
        ),
        IconButton(onPressed: () => Navigator.pop(context), icon: Icon(LucideIcons.x, color: Color(themeState.textMuted))),
      ],
    );
  }

  Widget _section(NomiTheme themeState, String title, String content) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: TextStyle(color: Color(themeState.textMuted), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
        const SizedBox(height: 12),
        Text(content, style: TextStyle(color: Color(themeState.textMain).withValues(alpha: 0.8), fontSize: 14, height: 1.6)),
      ],
    );
  }

  Widget _buildIntents(NomiTheme themeState) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('ACTIVATION INTENTS', style: TextStyle(color: Color(themeState.textMuted), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: widget.skill.intents.map((i) => Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            decoration: BoxDecoration(
              color: Color(themeState.textMain).withValues(alpha: 0.05),
              borderRadius: BorderRadius.circular(6),
            ),
            child: Text(i, style: TextStyle(color: Color(themeState.textMain).withValues(alpha: 0.8), fontSize: 11, fontFamily: 'monospace')),
          )).toList(),
        ),
      ],
    );
  }

  Widget _buildToggleSection(NomiTheme themeState, String title, bool isExpanded, Function(bool) onToggle, String content, {bool isCode = false}) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        GestureDetector(
          onTap: () => onToggle(!isExpanded),
          child: Row(
            children: [
              Text(title, style: TextStyle(color: Color(themeState.textMuted), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
              const Spacer(),
              Icon(isExpanded ? LucideIcons.chevronUp : LucideIcons.chevronDown, size: 14, color: Color(themeState.textMuted)),
            ],
          ),
        ),
        if (isExpanded)
          Container(
            margin: const EdgeInsets.only(top: 12),
            width: double.infinity,
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: themeState.isDark ? Colors.black : Color(themeState.bgHeader),
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
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
