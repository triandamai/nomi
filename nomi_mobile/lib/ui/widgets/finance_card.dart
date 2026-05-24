import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:intl/intl.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/core/config.dart';

class FinanceCard extends ConsumerStatefulWidget {
  final String refId;

  const FinanceCard({super.key, required this.refId});

  @override
  ConsumerState<FinanceCard> createState() => _FinanceCardState();
}

class _FinanceCardState extends ConsumerState<FinanceCard> {
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
      final response = await ref.read(apiClientProvider).dio.get('/money/history/${widget.refId}');
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
          color: const Color(AppConfig.deepSlate).withAlpha(153),
          borderRadius: BorderRadius.circular(20),
          border: Border.all(color: Colors.white10),
        ),
        child: const Center(child: SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2))),
      );
    }

    if (_error != null || _data == null) {
      return const SizedBox.shrink();
    }

    final String merchant = _data!['merchant_name'] ?? 'Unknown Merchant';
    final double total = (_data!['total_amount'] ?? 0).toDouble();
    final String category = _data!['category'] ?? 'General';
    final List items = _data!['items'] ?? [];

    final currencyFormat = NumberFormat.currency(
      locale: 'id-ID',
      symbol: 'Rp ',
      decimalDigits: 0,
    );

    const Color indigoColor = Color(AppConfig.indigo);

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 8),
      decoration: BoxDecoration(
        color: const Color(0xFF0f172a).withAlpha(153),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: indigoColor.withAlpha(77)),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withAlpha(51),
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
                color: indigoColor.withAlpha(25),
                border: Border(bottom: BorderSide(color: Colors.white.withAlpha(13))),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const Row(
                    children: [
                      Icon(LucideIcons.wallet, size: 12, color: indigoColor),
                      SizedBox(width: 8),
                      Text(
                        'FINANCE ENTRY',
                        style: TextStyle(
                          color: indigoColor,
                          fontSize: 9,
                          fontWeight: FontWeight.w900,
                          letterSpacing: 1.2,
                        ),
                      ),
                    ],
                  ),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                    decoration: BoxDecoration(
                      color: indigoColor.withAlpha(51),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: Text(
                      category.toUpperCase(),
                      style: const TextStyle(color: indigoColor, fontSize: 8, fontWeight: FontWeight.bold),
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
                    style: TextStyle(color: Colors.white38, fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1),
                  ),
                  Text(
                    merchant,
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
                            style: TextStyle(color: Colors.white38, fontSize: 8, fontWeight: FontWeight.w900, letterSpacing: 1),
                          ),
                          Text(
                            currencyFormat.format(total),
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
                          color: indigoColor.withAlpha(25),
                          borderRadius: BorderRadius.circular(16),
                          border: Border.all(color: indigoColor.withAlpha(25)),
                        ),
                        child: const Icon(LucideIcons.trendingDown, size: 20, color: indigoColor),
                      ),
                    ],
                  ),

                  // Line Items
                  if (items.isNotEmpty) ...[
                    const SizedBox(height: 20),
                    Container(
                      padding: const EdgeInsets.only(top: 12),
                      decoration: BoxDecoration(
                        border: Border(top: BorderSide(color: Colors.white.withAlpha(13))),
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
                                    "${item['name']} (x${item['quantity']})",
                                    style: TextStyle(color: Colors.white.withAlpha(127), fontSize: 11),
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                ),
                                Text(
                                  currencyFormat.format(item['total_amount']),
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
