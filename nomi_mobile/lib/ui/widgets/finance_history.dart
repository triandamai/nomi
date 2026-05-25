import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/repositories.dart';
import 'package:nomi_mobile/data/models/transaction.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;
import 'package:intl/intl.dart';
import 'dart:ui';

class FinanceHistorySheet extends ConsumerStatefulWidget {
  const FinanceHistorySheet({super.key});

  @override
  ConsumerState<FinanceHistorySheet> createState() => _FinanceHistorySheetState();
}

class _FinanceHistorySheetState extends ConsumerState<FinanceHistorySheet> {
  final _textController = TextEditingController();
  String? _selectedCategory;
  
  final List<Map<String, String>> _categories = [
    {'icon': '🍔', 'name': 'Food'},
    {'icon': '⛽', 'name': 'Fuel'},
    {'icon': '🛒', 'name': 'Shopping'},
    {'icon': '🏔️', 'name': 'Leisure'},
  ];

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(chatRepositoryProvider).syncTransactions();
    });
  }

  @override
  void dispose() {
    _textController.dispose();
    super.dispose();
  }

  void _syncWithFilters() {
    ref.read(chatRepositoryProvider).syncTransactions(
      query: _textController.text,
      category: _selectedCategory,
    );
  }

  @override
  Widget build(BuildContext context) {
    final transactionsStream = ref.watch(transactionsStreamProvider((_selectedCategory, _textController.text)));
    final size = MediaQuery.of(context).size;
    final bool isLargeScreen = size.width >= 700;

    return Container(
      width: double.infinity,
      constraints: BoxConstraints(maxHeight: size.height * 0.9),
      decoration: BoxDecoration(
        color: const Color(AppConfig.deepSlate).withValues(alpha: 0.95),
        border: Border(top: BorderSide(color: Colors.white.withValues(alpha: 0.1))),
      ),
      child: Column(
        children: [
          // Header
          ClipRRect(
            child: BackdropFilter(
              filter: ImageFilter.blur(sigmaX: 10, sigmaY: 10),
              child: Container(
                padding: const EdgeInsets.all(24),
                decoration: BoxDecoration(
                  color: Colors.white.withValues(alpha: 0.02),
                  border: Border(bottom: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
                ),
                child: Column(
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        const Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'FINANCIAL OPERATIONS',
                              style: TextStyle(
                                color: Color(AppConfig.emerald),
                                fontSize: 10,
                                fontWeight: FontWeight.w900,
                                letterSpacing: 2,
                              ),
                            ),
                            SizedBox(height: 4),
                            Text(
                              'Money Tracking',
                              style: TextStyle(color: Colors.white, fontSize: 22, fontWeight: FontWeight.bold),
                            ),
                          ],
                        ),
                        IconButton(
                          onPressed: () => Navigator.pop(context),
                          icon: const Icon(LucideIcons.x, color: Colors.white38),
                        ),
                      ],
                    ),
                    const SizedBox(height: 24),
                    
                    // Search Bar
                    Container(
                      decoration: BoxDecoration(
                        color: Colors.black.withValues(alpha: 0.3),
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: Colors.white.withValues(alpha: 0.05)),
                      ),
                      child: TextField(
                        controller: _textController,
                        onChanged: (_) => _syncWithFilters(),
                        style: const TextStyle(color: Colors.white, fontSize: 14),
                        decoration: const InputDecoration(
                          hintText: 'Search merchant or description...',
                          hintStyle: TextStyle(color: Colors.white24, fontSize: 14),
                          prefixIcon: Icon(LucideIcons.search, size: 16, color: Colors.white24),
                          border: InputBorder.none,
                          contentPadding: EdgeInsets.symmetric(vertical: 14),
                        ),
                      ),
                    ),
                    const SizedBox(height: 16),
                    
                    // Categories
                    SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: Row(
                        children: _categories.map((cat) {
                          final isSelected = _selectedCategory == cat['name'];
                          return Padding(
                            padding: const EdgeInsets.only(right: 8),
                            child: FilterChip(
                              label: Text('${cat['icon']} ${cat['name']}'),
                              selected: isSelected,
                              onSelected: (val) {
                                setState(() => _selectedCategory = val ? cat['name'] : null);
                                _syncWithFilters();
                              },
                              backgroundColor: Colors.white.withValues(alpha: 0.03),
                              selectedColor: Colors.blue.withValues(alpha: 0.2),
                              labelStyle: TextStyle(
                                color: isSelected ? Colors.blue : Colors.white38,
                                fontSize: 11,
                                fontWeight: FontWeight.bold,
                              ),
                              shape: RoundedRectangleBorder(
                                borderRadius: BorderRadius.circular(20),
                                side: BorderSide(color: isSelected ? Colors.blue.withValues(alpha: 0.3) : Colors.white.withValues(alpha: 0.05)),
                              ),
                            ),
                          );
                        }).toList(),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),

          // Transaction List
          Expanded(
            child: transactionsStream.when(
              data: (items) {
                if (items.isEmpty) {
                  return Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(LucideIcons.dollarSign, size: 48, color: Colors.white.withValues(alpha: 0.05)),
                        const SizedBox(height: 16),
                        Text(
                          'No transactions found',
                          style: TextStyle(color: Colors.white.withValues(alpha: 0.2), fontSize: 14, fontWeight: FontWeight.bold),
                        ),
                      ],
                    ),
                  );
                }
                return ListView.builder(
                  padding: const EdgeInsets.all(24),
                  itemCount: items.length,
                  itemBuilder: (context, index) {
                    return _TransactionListItem(tx: Transaction.fromDb(items[index]));
                  },
                );
              },
              loading: () => const Center(child: CircularProgressIndicator(strokeWidth: 2)),
              error: (e, _) => Center(child: Text('Sync Error: $e', style: const TextStyle(color: Colors.red))),
            ),
          ),
        ],
      ),
    );
  }
}

class _TransactionListItem extends StatelessWidget {
  final Transaction tx;
  const _TransactionListItem({required this.tx});

  @override
  Widget build(BuildContext context) {
    final currencyFormat = NumberFormat.currency(locale: 'id_ID', symbol: 'Rp ', decimalDigits: 0);
    final amount = double.tryParse(tx.totalAmount) ?? 0.0;
    
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
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Text(
                      tx.merchantName ?? 'Unknown Merchant',
                      style: const TextStyle(color: Colors.white, fontSize: 14, fontWeight: FontWeight.bold),
                    ),
                    if (tx.category != null) ...[
                      const SizedBox(width: 8),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          color: Colors.white.withValues(alpha: 0.05),
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Text(
                          tx.category!.toUpperCase(),
                          style: const TextStyle(color: Colors.white38, fontSize: 7, fontWeight: FontWeight.w900, letterSpacing: 1),
                        ),
                      ),
                    ],
                  ],
                ),
                const SizedBox(height: 4),
                Text(
                  tx.description ?? 'No description',
                  style: TextStyle(color: Colors.white.withValues(alpha: 0.3), fontSize: 12),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                const SizedBox(height: 8),
                Text(
                  DateFormat('MMM d, HH:mm').format(DateTime.parse(tx.createdAt)),
                  style: TextStyle(color: Colors.white.withValues(alpha: 0.15), fontSize: 9, fontWeight: FontWeight.bold, fontFamily: 'monospace'),
                ),
              ],
            ),
          ),
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Text(
                currencyFormat.format(amount),
                style: const TextStyle(color: Color(AppConfig.rose), fontSize: 14, fontWeight: FontWeight.w900, fontFamily: 'monospace'),
              ),
              const SizedBox(height: 4),
              if (tx.userDisplayName != null)
                Text(
                  '@${tx.userDisplayName}',
                  style: TextStyle(color: Colors.blue.withValues(alpha: 0.4), fontSize: 8, fontWeight: FontWeight.bold, fontStyle: FontStyle.italic),
                ),
            ],
          ),
        ],
      ),
    );
  }
}

final transactionsStreamProvider = StreamProvider.family<List<db.Transaction>, (String?, String?)>((ref, params) {
  return ref.watch(chatRepositoryProvider).watchTransactions(category: params.$1, search: params.$2);
});
