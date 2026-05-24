import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/core/config.dart';

class PluginProposalCard extends ConsumerStatefulWidget {
  final String refId;
  final VoidCallback? onOpenConsole;

  const PluginProposalCard({super.key, required this.refId, this.onOpenConsole});

  @override
  ConsumerState<PluginProposalCard> createState() => _PluginProposalCardState();
}

class _PluginProposalCardState extends ConsumerState<PluginProposalCard> {
  Map<String, dynamic>? _data;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _fetchDetail();
  }

  Future<void> _fetchDetail() async {
    try {
      final response = await ref.read(apiClientProvider).dio.get('/srp/proposals/${widget.refId}');
      if (!mounted) return;
      if (response.data != null && response.data['meta']['code'] == 200) {
        setState(() {
          _data = response.data['data'];
          _isLoading = false;
        });
      } else {
        setState(() {
          _error = "Not found";
          _isLoading = false;
        });
      }
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return Container(
        margin: const EdgeInsets.symmetric(vertical: 8),
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: Colors.white.withAlpha(13),
          borderRadius: BorderRadius.circular(20),
          border: Border.all(color: Colors.white10),
        ),
        child: const Center(child: SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2))),
      );
    }

    if (_error != null || _data == null) {
      return const SizedBox.shrink();
    }

    final String name = _data!['name'] ?? 'New Skill';
    final String status = _data!['status'] ?? 'pending';
    final String description = _data!['description'] ?? '';

    const Color emeraldColor = Color(AppConfig.emerald);
    const Color amberColor = Color(AppConfig.amber);
    const Color roseColor = Color(AppConfig.rose);

    Color statusColor;
    switch (status.toLowerCase()) {
      case 'ready': statusColor = emeraldColor; break;
      case 'processing': statusColor = amberColor; break;
      case 'failed': statusColor = roseColor; break;
      default: statusColor = const Color(0xFF94a3b8);
    }

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        color: Colors.white.withAlpha(13),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: Colors.white.withAlpha(25)),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withAlpha(51),
            blurRadius: 15,
            offset: const Offset(0, 5),
          ),
        ],
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(20),
        child: Column(
          children: [
            // Header
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              decoration: BoxDecoration(
                color: Colors.white.withAlpha(13),
                border: Border(bottom: BorderSide(color: Colors.white.withAlpha(13))),
              ),
              child: const Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Row(
                    children: [
                      Icon(LucideIcons.factory, size: 12, color: emeraldColor),
                      SizedBox(width: 8),
                      Text(
                        'AUTONOMOUS BLUEPRINT',
                        style: TextStyle(
                          color: emeraldColor,
                          fontSize: 8,
                          fontWeight: FontWeight.w900,
                          letterSpacing: 1.5,
                        ),
                      ),
                    ],
                  ),
                  Icon(LucideIcons.sparkles, size: 12, color: emeraldColor),
                ],
              ),
            ),

            // Body
            Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Container(
                        padding: const EdgeInsets.all(10),
                        decoration: BoxDecoration(
                          color: const Color(0xFF020617),
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(color: Colors.white.withAlpha(13)),
                        ),
                        child: const Icon(LucideIcons.cpu, size: 24, color: emeraldColor),
                      ),
                      const SizedBox(width: 16),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Row(
                              mainAxisAlignment: MainAxisAlignment.spaceBetween,
                              children: [
                                const Text(
                                  'TARGET SKILL',
                                  style: TextStyle(color: Colors.white38, fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1),
                                ),
                                Container(
                                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                                  decoration: BoxDecoration(
                                    color: statusColor.withAlpha(25),
                                    borderRadius: BorderRadius.circular(4),
                                  ),
                                  child: Text(
                                    status.toUpperCase(),
                                    style: TextStyle(color: statusColor, fontSize: 7, fontWeight: FontWeight.bold),
                                  ),
                                ),
                              ],
                            ),
                            Text(
                              name,
                              style: const TextStyle(color: Colors.white, fontSize: 14, fontWeight: FontWeight.bold, fontFamily: 'monospace'),
                              overflow: TextOverflow.ellipsis,
                            ),
                          ],
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  Text(
                    description,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(color: Colors.white.withAlpha(127), fontSize: 11, height: 1.4, fontStyle: FontStyle.italic),
                  ),
                  const SizedBox(height: 16),
                  
                  // Action Button
                  SizedBox(
                    width: double.infinity,
                    child: ElevatedButton(
                      onPressed: widget.onOpenConsole,
                      style: ElevatedButton.styleFrom(
                        backgroundColor: emeraldColor,
                        foregroundColor: const Color(0xFF020617),
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                        padding: const EdgeInsets.symmetric(vertical: 12),
                        elevation: 0,
                      ),
                      child: const Row(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Text(
                            'OPEN FACTORY CONSOLE',
                            style: TextStyle(fontWeight: FontWeight.w900, fontSize: 9, letterSpacing: 1),
                          ),
                          SizedBox(width: 8),
                          Icon(LucideIcons.arrowRight, size: 14),
                        ],
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
