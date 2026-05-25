import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/providers/navigation_provider.dart';
import 'package:nomi_mobile/data/models/admin_conversation.dart';
import 'package:nomi_mobile/core/utils/formatter.dart';

class MonitorPage extends ConsumerStatefulWidget {
  const MonitorPage({super.key});

  @override
  ConsumerState<MonitorPage> createState() => _MonitorPageState();
}

class _MonitorPageState extends ConsumerState<MonitorPage> {
  List<AdminConversation> _sessions = [];
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _fetchSessions();
  }

  Future<void> _fetchSessions() async {
    if (!mounted) return;
    setState(() => _isLoading = true);
    try {
      final data = await ref.read(chatRepositoryProvider).getAdminConversations();
      if (mounted) {
        setState(() {
          _sessions = data;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) setState(() => _isLoading = false);
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
            title: const Text('Conversation Monitor', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
          ),
      body: Column(
        children: [
          _buildHeader(),
          Expanded(
            child: _isLoading 
              ? const Center(child: CircularProgressIndicator())
              : _sessions.isEmpty
                ? const Center(child: Text('No active sessions detected.', style: TextStyle(color: Colors.white38)))
                : ListView.builder(
                    padding: const EdgeInsets.all(24),
                    itemCount: _sessions.length,
                    itemBuilder: (context, index) => _SessionItem(
                      session: _sessions[index],
                      onTap: () => _showDetailSheet(_sessions[index]),
                    ),
                  ),
          ),
        ],
      ),
    );
  }

  void _showDetailSheet(AdminConversation session) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => _SessionDetailSheet(
        session: session,
        onUpdate: _fetchSessions,
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
          const Icon(LucideIcons.lineChart, color: Color(AppConfig.blue), size: 24),
          const SizedBox(width: 16),
          const Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('CONVERSATION MONITOR', style: TextStyle(color: Colors.white, fontSize: 16, fontWeight: FontWeight.bold)),
              Text('Cross-System Session Observability', style: TextStyle(color: Colors.white38, fontSize: 10, fontWeight: FontWeight.w500)),
            ],
          ),
          const Spacer(),
          IconButton(
            onPressed: _fetchSessions,
            icon: const Icon(LucideIcons.refreshCw, size: 18, color: Colors.white38),
          ),
        ],
      ),
    );
  }
}

class _SessionItem extends StatelessWidget {
  final AdminConversation session;
  final VoidCallback onTap;
  
  const _SessionItem({required this.session, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final double usagePercent = (session.maxTokenUsage != null && session.maxTokenUsage! > 0)
        ? (session.cumulativeTokens ?? 0) / session.maxTokenUsage!
        : 0.0;
    
    final Color progressColor = usagePercent > 0.9 
        ? const Color(AppConfig.rose) 
        : usagePercent > 0.7 
            ? const Color(AppConfig.amber) 
            : const Color(AppConfig.blue);

    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.03),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
      ),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(20),
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            children: [
              Row(
                children: [
                  Container(
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: const Color(AppConfig.blue).withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(16),
                    ),
                    child: const Icon(LucideIcons.messagesSquare, size: 20, color: Color(AppConfig.blue)),
                  ),
                  const SizedBox(width: 20),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(session.title ?? 'Untitled Session', style: const TextStyle(color: Colors.white, fontSize: 15, fontWeight: FontWeight.bold)),
                        const SizedBox(height: 4),
                        Text(session.id, style: const TextStyle(color: Colors.white24, fontSize: 10, fontFamily: 'monospace')),
                      ],
                    ),
                  ),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.end,
                    children: [
                      Text(
                        '${Formatter.formatTokenCount(session.cumulativeTokens ?? 0)} TOKENS',
                        style: TextStyle(color: progressColor, fontSize: 11, fontWeight: FontWeight.w900, letterSpacing: 1),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        'LIMIT: ${Formatter.formatTokenCount(session.maxTokenUsage ?? 0)}',
                        style: const TextStyle(color: Colors.white24, fontSize: 9, fontWeight: FontWeight.bold),
                      ),
                    ],
                  ),
                ],
              ),
              const SizedBox(height: 20),
              ClipRRect(
                borderRadius: BorderRadius.circular(4),
                child: LinearProgressIndicator(
                  value: usagePercent.clamp(0.0, 1.0),
                  backgroundColor: Colors.white.withValues(alpha: 0.05),
                  valueColor: AlwaysStoppedAnimation<Color>(progressColor),
                  minHeight: 6,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _SessionDetailSheet extends ConsumerStatefulWidget {
  final AdminConversation session;
  final VoidCallback onUpdate;

  const _SessionDetailSheet({required this.session, required this.onUpdate});

  @override
  ConsumerState<_SessionDetailSheet> createState() => _SessionDetailSheetState();
}

class _SessionDetailSheetState extends ConsumerState<_SessionDetailSheet> {
  late final TextEditingController _limitController;
  late final TextEditingController _titleController;
  bool _isSaving = false;

  @override
  void initState() {
    super.initState();
    _limitController = TextEditingController(text: widget.session.maxTokenUsage?.toString() ?? '0');
    _titleController = TextEditingController(text: widget.session.title ?? '');
  }

  @override
  void dispose() {
    _limitController.dispose();
    _titleController.dispose();
    super.dispose();
  }

  Future<void> _handleSave() async {
    final int? newLimit = int.tryParse(_limitController.text);
    if (newLimit == null) return;

    setState(() => _isSaving = true);
    final success = await ref.read(chatRepositoryProvider).updateAdminConversation(
      widget.session.id,
      maxTokenUsage: newLimit,
      title: _titleController.text,
    );

    if (mounted) {
      setState(() => _isSaving = false);
      if (success) {
        widget.onUpdate();
        Navigator.pop(context);
      } else {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Failed to update session parameters.')),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40),
        child: Container(
          padding: EdgeInsets.fromLTRB(24, 24, 24, MediaQuery.of(context).viewInsets.bottom + 24),
          decoration: BoxDecoration(
            color: const Color(AppConfig.deepSlate).withValues(alpha: 0.9),
            borderRadius: const BorderRadius.vertical(top: Radius.circular(24)),
            border: const Border(top: BorderSide(color: Colors.white10)),
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('SESSION PARAMETERS', style: TextStyle(color: Color(AppConfig.blue), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
                      SizedBox(height: 4),
                      Text('Adjust Constraints', style: TextStyle(color: Colors.white, fontSize: 20, fontWeight: FontWeight.bold)),
                    ],
                  ),
                  IconButton(onPressed: () => Navigator.pop(context), icon: const Icon(LucideIcons.x, color: Colors.white38)),
                ],
              ),
              const SizedBox(height: 32),
              _buildField('Session Title', _titleController, 'e.g. Technical Research Pass'),
              const SizedBox(height: 24),
              _buildField('Max Token Limit', _limitController, 'e.g. 500000', isNumeric: true),
              const SizedBox(height: 40),
              SizedBox(
                width: double.infinity,
                height: 56,
                child: ElevatedButton(
                  onPressed: _isSaving ? null : _handleSave,
                  style: ElevatedButton.styleFrom(backgroundColor: const Color(AppConfig.blue), foregroundColor: Colors.white, shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)), elevation: 0),
                  child: _isSaving 
                    ? const SizedBox(width: 24, height: 24, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                    : const Text('UPDATE SESSION', style: TextStyle(fontWeight: FontWeight.w900, letterSpacing: 1)),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildField(String label, TextEditingController controller, String hint, {bool isNumeric = false}) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label.toUpperCase(), style: const TextStyle(color: Colors.white38, fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1)),
        const SizedBox(height: 12),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          decoration: BoxDecoration(color: Colors.white10, borderRadius: BorderRadius.circular(12), border: Border.all(color: Colors.white.withValues(alpha: 0.05))),
          child: TextField(controller: controller, keyboardType: isNumeric ? TextInputType.number : TextInputType.text, style: const TextStyle(color: Colors.white, fontSize: 14), decoration: InputDecoration(hintText: hint, hintStyle: const TextStyle(color: Colors.white24, fontSize: 14), border: InputBorder.none)),
        ),
      ],
    );
  }
}
