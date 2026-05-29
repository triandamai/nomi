import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
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
            title: Text('Conversation Monitor', style: TextStyle(color: Color(themeState.textMain), fontSize: 18, fontWeight: FontWeight.bold)),
          ),
      body: Column(
        children: [
          _buildHeader(themeState),
          Expanded(
            child: _isLoading 
              ? const Center(child: CircularProgressIndicator())
              : _sessions.isEmpty
                ? Center(child: Text('No active sessions detected.', style: TextStyle(color: Color(themeState.textMuted))))
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
          Icon(LucideIcons.lineChart, color: Color(themeState.primaryColor), size: 24),
          const SizedBox(width: 16),
          Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('CONVERSATION MONITOR', style: TextStyle(color: Color(themeState.textMain), fontSize: 16, fontWeight: FontWeight.bold)),
              Text('Cross-System Session Observability', style: TextStyle(color: Color(themeState.textMuted), fontSize: 10, fontWeight: FontWeight.w500)),
            ],
          ),
          const Spacer(),
          IconButton(
            onPressed: _fetchSessions,
            icon: Icon(LucideIcons.refreshCw, size: 18, color: Color(themeState.textMuted)),
          ),
        ],
      ),
    );
  }
}

class _SessionItem extends ConsumerWidget {
  final AdminConversation session;
  final VoidCallback onTap;
  
  const _SessionItem({required this.session, required this.onTap});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    final double usagePercent = (session.maxTokenUsage != null && session.maxTokenUsage! > 0)
        ? (session.cumulativeTokens ?? 0) / session.maxTokenUsage!
        : 0.0;
    
    final Color progressColor = usagePercent > 0.9 
        ? const Color(AppConfig.rose) 
        : usagePercent > 0.7 
            ? const Color(AppConfig.amber) 
            : Color(themeState.primaryColor);

    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      decoration: BoxDecoration(
        color: Color(themeState.textMain).withValues(alpha: 0.03),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
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
                      color: Color(themeState.primaryColor).withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(16),
                    ),
                    child: Icon(LucideIcons.messagesSquare, size: 20, color: Color(themeState.primaryColor)),
                  ),
                  const SizedBox(width: 20),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(session.title ?? 'Untitled Session', style: TextStyle(color: Color(themeState.textMain), fontSize: 15, fontWeight: FontWeight.bold)),
                        const SizedBox(height: 4),
                        Text(session.id, style: TextStyle(color: Color(themeState.textMuted), fontSize: 10, fontFamily: 'monospace')),
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
                        style: TextStyle(color: Color(themeState.textMuted), fontSize: 9, fontWeight: FontWeight.bold),
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
                  backgroundColor: Color(themeState.textMain).withValues(alpha: 0.05),
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
  late double _interactionGate;
  late double _intentClassification;
  late double _guardrails;
  bool _isSaving = false;

  @override
  void initState() {
    super.initState();
    _limitController = TextEditingController(text: widget.session.maxTokenUsage?.toString() ?? '0');
    _titleController = TextEditingController(text: widget.session.title ?? '');
    
    final thresholds = widget.session.gatewayThresholds ?? {};
    _interactionGate = (thresholds['interaction_gate'] as num?)?.toDouble() ?? 0.60;
    _intentClassification = (thresholds['intent_classification'] as num?)?.toDouble() ?? 0.40;
    _guardrails = (thresholds['guardrails'] as num?)?.toDouble() ?? 0.65;
  }

  @override
  void dispose() {
    _limitController.dispose();
    _titleController.dispose();
    super.dispose();
  }

  (String, Color, String) _getInteractionMode(NomiTheme themeState, double val) {
    if (val <= 0.25) return ('Proactive', const Color(AppConfig.emerald), '🏁');
    if (val <= 0.50) return ('Balanced', Color(themeState.primaryColor), '🤝');
    if (val <= 0.75) return ('Conservative', const Color(AppConfig.amber), '🛡️');
    return ('Silent Monitor', Color(themeState.textMuted), '🤫');
  }

  (String, Color, String) _getIntentMode(NomiTheme themeState, double val) {
    if (val <= 0.40) return ('Experimental', const Color(AppConfig.indigo), '🧪');
    if (val <= 0.70) return ('Adaptive', Color(themeState.primaryColor), '🏎️');
    return ('Strict', const Color(AppConfig.rose), '📐');
  }

  (String, Color, String) _getGuardrailMode(NomiTheme themeState, double val) {
    if (val <= 0.50) return ('Permissive', const Color(AppConfig.emerald), '🔓');
    if (val <= 0.80) return ('Standard', Color(themeState.primaryColor), '👤');
    return ('Hardened Shield', const Color(AppConfig.rose), '🌋');
  }

  Future<void> _handleSave() async {
    final int? newLimit = int.tryParse(_limitController.text);
    if (newLimit == null) return;

    setState(() => _isSaving = true);
    final success = await ref.read(chatRepositoryProvider).updateAdminConversation(
      widget.session.id,
      maxTokenUsage: newLimit,
      title: _titleController.text,
      thresholds: {
        'interaction_gate': _interactionGate,
        'intent_classification': _intentClassification,
        'guardrails': _guardrails,
      },
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
    final themeState = ref.watch(themeProvider);
    return ClipRRect(
      borderRadius: const BorderRadius.only(
        topLeft: Radius.circular(20),
        topRight: Radius.circular(20),
      ),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: Container(
          padding: EdgeInsets.fromLTRB(24, 24, 24, MediaQuery.of(context).viewInsets.bottom + 40),
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
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('SESSION PARAMETERS', style: TextStyle(color: Color(themeState.primaryColor), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
                        const SizedBox(height: 4),
                        Text('Adjust Constraints', style: TextStyle(color: Color(themeState.textMain), fontSize: 20, fontWeight: FontWeight.bold)),
                      ],
                    ),
                    IconButton(onPressed: () => Navigator.pop(context), icon: Icon(LucideIcons.x, color: Color(themeState.textMuted))),
                  ],
                ),
                const SizedBox(height: 32),
                _buildField(themeState, 'Session Title', _titleController, 'e.g. Technical Research Pass'),
                const SizedBox(height: 24),
                _buildField(themeState, 'Max Token Limit', _limitController, 'e.g. 500000', isNumeric: true),
                
                const SizedBox(height: 32),
                Divider(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
                const SizedBox(height: 24),
                Text('BEHAVIOR BOUNDARIES (DEB)', style: TextStyle(color: Color(themeState.textMuted), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
                const SizedBox(height: 24),
                
                _buildSlider(
                  'Sociability', 
                  _interactionGate, 
                  _getInteractionMode(themeState, _interactionGate),
                  (val) => setState(() => _interactionGate = val)
                ),
                const SizedBox(height: 24),
                _buildSlider(
                  'Confidence', 
                  _intentClassification, 
                  _getIntentMode(themeState, _intentClassification),
                  (val) => setState(() => _intentClassification = val)
                ),
                const SizedBox(height: 24),
                _buildSlider(
                  'Vigilance', 
                  _guardrails, 
                  _getGuardrailMode(themeState, _guardrails),
                  (val) => setState(() => _guardrails = val)
                ),

                const SizedBox(height: 40),
                SizedBox(
                  width: double.infinity,
                  height: 56,
                  child: ElevatedButton(
                    onPressed: _isSaving ? null : _handleSave,
                    style: ElevatedButton.styleFrom(
                      backgroundColor: Color(themeState.primaryColor), 
                      foregroundColor: Colors.white, 
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)), 
                      elevation: 0
                    ),
                    child: _isSaving 
                      ? const SizedBox(width: 24, height: 24, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                      : const Text('UPDATE SESSION', style: TextStyle(fontWeight: FontWeight.w900, letterSpacing: 1)),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildSlider(String label, double value, (String, Color, String) mode, Function(double) onChanged) {
    final themeState = ref.watch(themeProvider);
    return Column(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(label, style: TextStyle(color: Color(themeState.textMain).withValues(alpha: 0.8), fontSize: 11, fontWeight: FontWeight.bold)),
            Text('${mode.$3} ${mode.$1} (${value.toStringAsFixed(2)})', style: TextStyle(color: mode.$2, fontSize: 10, fontWeight: FontWeight.w900, fontFamily: 'monospace')),
          ],
        ),
        const SizedBox(height: 8),
        SliderTheme(
          data: SliderTheme.of(context).copyWith(
            trackHeight: 2,
            activeTrackColor: mode.$2,
            inactiveTrackColor: Color(themeState.borderMain).withValues(alpha: 0.5),
            thumbColor: Color(themeState.textMain),
            overlayColor: mode.$2.withValues(alpha: 0.1),
            thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 6),
          ),
          child: Slider(
            value: value,
            onChanged: onChanged,
          ),
        ),
      ],
    );
  }

  Widget _buildField(NomiTheme themeState, String label, TextEditingController controller, String hint, {bool isNumeric = false}) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label.toUpperCase(), style: TextStyle(color: Color(themeState.textMuted), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1)),
        const SizedBox(height: 12),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          decoration: BoxDecoration(
            color: Color(themeState.textMain).withValues(alpha: 0.03), 
            borderRadius: BorderRadius.circular(12), 
            border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5))
          ),
          child: TextField(
            controller: controller, 
            keyboardType: isNumeric ? TextInputType.number : TextInputType.text, 
            style: TextStyle(color: Color(themeState.textMain), fontSize: 14), 
            decoration: InputDecoration(
              hintText: hint, 
              hintStyle: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.5), fontSize: 14), 
              border: InputBorder.none
            )
          ),
        ),
      ],
    );
  }
}
