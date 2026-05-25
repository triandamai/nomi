import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:intl/intl.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/data/models/transaction.dart';

class FinanceCard extends ConsumerStatefulWidget {
  final String refId;

  const FinanceCard({super.key, required this.refId});

  @override
  ConsumerState<FinanceCard> createState() => _FinanceCardState();
}

class _FinanceCardState extends ConsumerState<FinanceCard> {
  Transaction? _transaction;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _fetchDetail();
  }

  Future<void> _fetchDetail() async {
    try {
      final tx = await ref.read(chatRepositoryProvider).getTransaction(widget.refId);
      if (!mounted) return;
      
      if (tx != null) {
        setState(() {
          _transaction = tx;
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
          color: Colors.white.withValues(alpha: 0.03),
          borderRadius: BorderRadius.circular(20),
          border: Border.all(color: Colors.white10),
        ),
        child: const Center(child: SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2))),
      );
    }

    if (_error != null || _transaction == null) {
      return const SizedBox.shrink();
    }

    final currencyFormat = NumberFormat.currency(
      locale: 'id-ID',
      symbol: 'Rp ',
      decimalDigits: 0,
    );

    final amount = double.tryParse(_transaction!.totalAmount) ?? 0.0;
    final items = _transaction!.items ?? [];

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.03),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: const Color(AppConfig.emerald).withValues(alpha: 0.2)),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.2),
            blurRadius: 10,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Header
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              decoration: BoxDecoration(
                color: const Color(AppConfig.emerald).withValues(alpha: 0.05),
                border: Border(bottom: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Row(
                    children: [
                      const Icon(LucideIcons.wallet, size: 12, color: Color(AppConfig.emerald)),
                      const SizedBox(width: 8),
                      Text(
                        'FINANCE ENTRY',
                        style: TextStyle(
                          color: const Color(AppConfig.emerald).withValues(alpha: 0.8),
                          fontSize: 9,
                          fontWeight: FontWeight.w900,
                          letterSpacing: 1.2,
                        ),
                      ),
                    ],
                  ),
                  if (_transaction!.category != null)
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                      decoration: BoxDecoration(
                        color: const Color(AppConfig.emerald).withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: Text(
                        _transaction!.category!.toUpperCase(),
                        style: const TextStyle(color: Color(AppConfig.emerald), fontSize: 8, fontWeight: FontWeight.bold),
                      ),
                    ),
                ],
              ),
            ),

            // Content
            Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    'MERCHANT',
                    style: TextStyle(color: Colors.white24, fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1),
                  ),
                  Text(
                    _transaction!.merchantName ?? 'Unknown Merchant',
                    style: const TextStyle(color: Colors.white, fontSize: 16, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 16),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          const Text(
                            'TOTAL AMOUNT',
                            style: TextStyle(color: Colors.white24, fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1),
                          ),
                          Text(
                            currencyFormat.format(amount),
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 20,
                              fontWeight: FontWeight.w900,
                              letterSpacing: -0.5,
                            ),
                          ),
                        ],
                      ),
                      Container(
                        padding: const EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: const Color(AppConfig.emerald).withValues(alpha: 0.1),
                          borderRadius: BorderRadius.circular(16),
                          border: Border.all(color: const Color(AppConfig.emerald).withValues(alpha: 0.1)),
                        ),
                        child: const Icon(LucideIcons.trendingDown, size: 20, color: Color(AppConfig.emerald)),
                      ),
                    ],
                  ),

                  // Line Items
                  if (items.isNotEmpty) ...[
                    const SizedBox(height: 20),
                    Container(
                      padding: const EdgeInsets.only(top: 12),
                      decoration: BoxDecoration(
                        border: Border(top: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          const Row(
                            children: [
                              Icon(LucideIcons.receipt, size: 10, color: Colors.white24),
                              SizedBox(width: 8),
                              Text(
                                'LINE ITEMS',
                                style: TextStyle(color: Colors.white24, fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1),
                              ),
                            ],
                          ),
                          const SizedBox(height: 8),
                          ...items.take(3).map((item) => Padding(
                            padding: const EdgeInsets.only(bottom: 4),
                            child: Row(
                              mainAxisAlignment: MainAxisAlignment.spaceBetween,
                              children: [
                                Expanded(
                                  child: Text(
                                    "${item.name} (x${item.quantity})",
                                    style: TextStyle(color: Colors.white.withValues(alpha: 0.5), fontSize: 11),
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                ),
                                Text(
                                  currencyFormat.format(double.tryParse(item.totalAmount) ?? 0.0),
                                  style: const TextStyle(color: Colors.white, fontSize: 11, fontWeight: FontWeight.bold, fontFamily: 'monospace'),
                                ),
                              ],
                            ),
                          )),
                          if (items.length > 3)
                            Padding(
                              padding: const EdgeInsets.only(top: 4),
                              child: Text(
                                "+${items.length - 3} more items...",
                                style: TextStyle(color: Colors.white24, fontSize: 9, fontStyle: FontStyle.italic),
                              ),
                            ),
                        ],
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
