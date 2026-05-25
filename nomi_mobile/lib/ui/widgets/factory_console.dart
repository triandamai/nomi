import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
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
    final size = MediaQuery.of(context).size;
    final isLargeScreen = size.width >= 700;

    return ClipRRect(
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 40, sigmaY: 40),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: size.height * 0.9),
          decoration: BoxDecoration(
            gradient: LinearGradient(
              begin: Alignment.topLeft,
              end: Alignment.bottomRight,
              colors: [
                const Color(AppConfig.deepSlate).withValues(alpha: 0.7),
                const Color(0xFF1e293b).withValues(alpha: 0.4),
              ],
            ),
            border: const Border(top: BorderSide(color: Colors.white10)),
          ),
          padding: EdgeInsets.symmetric(horizontal: 24, vertical: isLargeScreen ? 24 : 32),
          child: SafeArea(
            child: Column(
              children: [
                // Header
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('FACTORY CONSOLE', style: TextStyle(color: Color(AppConfig.emerald), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
                        SizedBox(height: 4),
                        Text('Agentic Pipelines', style: TextStyle(color: Colors.white, fontSize: 22, fontWeight: FontWeight.bold)),
                      ],
                    ),
                    IconButton(
                      onPressed: () => Navigator.pop(context),
                      icon: const Icon(LucideIcons.x, color: Colors.white38),
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

class _ProposalItem extends StatelessWidget {
  final SrpProposal proposal;
  const _ProposalItem({required this.proposal});

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.03),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: Colors.indigo.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(16),
            ),
            child: const Icon(LucideIcons.cpu, size: 20, color: Colors.indigo),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(proposal.name, style: const TextStyle(color: Colors.white, fontSize: 14, fontWeight: FontWeight.bold)),
                Text(proposal.slug, style: const TextStyle(color: Colors.indigo, fontSize: 10, fontFamily: 'monospace')),
              ],
            ),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: proposal.status == 'approved' ? const Color(AppConfig.emerald).withValues(alpha: 0.1) : Colors.amber.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text(proposal.status.toUpperCase(), style: TextStyle(color: proposal.status == 'approved' ? const Color(AppConfig.emerald) : Colors.amber, fontSize: 8, fontWeight: FontWeight.w900)),
          ),

        ],
      ),
    );
  }
}
