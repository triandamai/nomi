import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/data/models/srp_proposal.dart';
import 'dart:ui';

class FactoryConsoleSheet extends ConsumerStatefulWidget {
  const FactoryConsoleSheet({super.key});

  @override
  ConsumerState<FactoryConsoleSheet> createState() => _FactoryConsoleSheetState();
}

class _FactoryConsoleSheetState extends ConsumerState<FactoryConsoleSheet> {
  List<SrpProposal> _proposals = [];
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _fetchProposals();
  }

  Future<void> _fetchProposals() async {
    try {
      final data = await ref.read(chatRepositoryProvider).getProposals();
      if (mounted) {
        setState(() {
          _proposals = data;
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
    final size = MediaQuery.of(context).size;
    final isLargeScreen = size.width >= 700;

    return ClipRRect(
      borderRadius: const BorderRadius.only(
        topLeft: Radius.circular(20),
        topRight: Radius.circular(20),
      ),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: size.height * 0.9),
          decoration: BoxDecoration(
            color: themeState.isDark 
              ? Color(themeState.slate950).withValues(alpha: 0.85) 
              : Color(themeState.bgHeader).withValues(alpha: 0.92),
            border: Border.all(
              color: Color(themeState.borderMain).withValues(alpha: 0.25),
              width: 1.2,
            ),
            borderRadius: const BorderRadius.only(
              topLeft: Radius.circular(20),
              topRight: Radius.circular(20),
            ),
          ),
          padding: EdgeInsets.symmetric(horizontal: 24, vertical: isLargeScreen ? 24 : 32),
          child: SafeArea(
            child: Column(
              children: [
                // Header
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'FACTORY CONSOLE', 
                          style: TextStyle(
                            color: Color(themeState.accentColor), 
                            fontSize: 10, 
                            fontWeight: FontWeight.w900, 
                            letterSpacing: 2
                          )
                        ),
                        const SizedBox(height: 4),
                        Text(
                          'Agentic Pipelines', 
                          style: TextStyle(
                            color: Color(themeState.textMain), 
                            fontSize: 22, 
                            fontWeight: FontWeight.bold
                          )
                        ),
                      ],
                    ),
                    IconButton(
                      onPressed: () => Navigator.pop(context),
                      icon: Icon(LucideIcons.x, color: Color(themeState.textMuted)),
                    ),
                  ],
                ),
                const SizedBox(height: 32),
                
                Expanded(
                  child: _isLoading 
                    ? const Center(child: CircularProgressIndicator())
                    : ListView.builder(
                        itemCount: _proposals.length,
                        itemBuilder: (context, index) => _ProposalItem(proposal: _proposals[index]),
                      ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _ProposalItem extends ConsumerWidget {
  final SrpProposal proposal;
  const _ProposalItem({required this.proposal});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    final isApproved = proposal.status == 'approved';
    
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Color(themeState.textMain).withValues(alpha: 0.03),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: Color(themeState.primaryColor).withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Icon(LucideIcons.cpu, size: 20, color: Color(themeState.primaryColor)),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  proposal.name, 
                  style: TextStyle(color: Color(themeState.textMain), fontSize: 14, fontWeight: FontWeight.bold)
                ),
                Text(
                  proposal.slug, 
                  style: TextStyle(color: Color(themeState.primaryColor), fontSize: 10, fontFamily: 'monospace')
                ),
              ],
            ),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: isApproved ? Color(themeState.accentColor).withValues(alpha: 0.1) : Colors.amber.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text(
              proposal.status.toUpperCase(), 
              style: TextStyle(
                color: isApproved ? Color(themeState.accentColor) : Colors.amber, 
                fontSize: 8, 
                fontWeight: FontWeight.w900
              )
            ),
          ),
        ],
      ),
    );
  }
}
