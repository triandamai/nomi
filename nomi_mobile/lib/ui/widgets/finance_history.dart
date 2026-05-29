import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
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
    final themeState = ref.watch(themeProvider);
    final transactionsStream = ref.watch(transactionsStreamProvider((_selectedCategory, _textController.text)));
    final size = MediaQuery.of(context).size;

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
          child: Column(
        children: [
          // Header
          ClipRRect(
            child: BackdropFilter(
              filter: ImageFilter.blur(sigmaX: 10, sigmaY: 10),
              child: Container(
                padding: const EdgeInsets.all(24),
                decoration: BoxDecoration(
                  color: Color(themeState.textMain).withValues(alpha: 0.02),
                  border: Border(bottom: BorderSide(color: Color(themeState.borderMain).withValues(alpha: 0.5))),
                ),
                child: Column(
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'FINANCIAL OPERATIONS',
                              style: TextStyle(
                                color: Color(themeState.accentColor),
                                fontSize: 10,
                                fontWeight: FontWeight.w900,
                                letterSpacing: 2,
                              ),
                            ),
                            const SizedBox(height: 4),
                            Text(
                              'Money Tracking',
                              style: TextStyle(color: Color(themeState.textMain), fontSize: 22, fontWeight: FontWeight.bold),
                            ),
                          ],
                        ),
                        IconButton(
                          onPressed: () => Navigator.pop(context),
                          icon: Icon(LucideIcons.x, color: Color(themeState.textMuted)),
                        ),
                      ],
                    ),
                    const SizedBox(height: 24),
                    
                    // Search Bar
                    Container(
                      decoration: BoxDecoration(
                        color: themeState.isDark ? Colors.black.withValues(alpha: 0.3) : Color(themeState.textMain).withValues(alpha: 0.05),
                        borderRadius: BorderRadius.circular(16),
                        border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
                      ),
                      child: TextField(
                        controller: _textController,
                        onChanged: (_) => _syncWithFilters(),
                        style: TextStyle(color: Color(themeState.textMain), fontSize: 14),
                        decoration: InputDecoration(
                          hintText: 'Search merchant or description...',
                          hintStyle: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.5), fontSize: 14),
                          prefixIcon: Icon(LucideIcons.search, size: 16, color: Color(themeState.textMuted)),
                          border: InputBorder.none,
                          contentPadding: const EdgeInsets.symmetric(vertical: 14),
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
                              backgroundColor: Color(themeState.textMain).withValues(alpha: 0.03),
                              selectedColor: Color(themeState.primaryColor).withValues(alpha: 0.2),
                              labelStyle: TextStyle(
                                color: isSelected ? Color(themeState.primaryColor) : Color(themeState.textMuted),
                                fontSize: 11,
                                fontWeight: FontWeight.bold,
                              ),
                              shape: RoundedRectangleBorder(
                                borderRadius: BorderRadius.circular(20),
                                side: BorderSide(color: isSelected ? Color(themeState.primaryColor).withValues(alpha: 0.3) : Color(themeState.borderMain).withValues(alpha: 0.5)),
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
                        Icon(LucideIcons.dollarSign, size: 48, color: Color(themeState.textMuted).withValues(alpha: 0.1)),
                        const SizedBox(height: 16),
                        Text(
                          'No transactions found',
                          style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.4), fontSize: 14, fontWeight: FontWeight.bold),
                        ),
                      ],
                    ),
                  );
                }
                return ListView.builder(
                  padding: const EdgeInsets.all(24),
                  itemCount: items.length,
                  itemBuilder: (context, index) {
                    final tx = Transaction.fromDb(items[index]);
                    return _TransactionListItem(
                      tx: tx,
                      onTap: () => _showTransactionDetail(context, tx),
                    );
                  },
                );
              },
              loading: () => const Center(child: CircularProgressIndicator(strokeWidth: 2)),
              error: (e, _) => Center(child: Text('Sync Error: $e', style: const TextStyle(color: Colors.red))),
            ),
          ),
        ],
      ),
    ),
    ),
    );
  }

  void _showTransactionDetail(BuildContext context, Transaction tx) {
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      backgroundColor: Colors.transparent,
      builder: (context) => _TransactionDetailSheet(tx: tx),
    );
  }
}

class _TransactionDetailSheet extends ConsumerWidget {
  final Transaction tx;
  const _TransactionDetailSheet({required this.tx});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    final currencyFormat = NumberFormat.currency(locale: 'id_ID', symbol: 'Rp ', decimalDigits: 0);
    final amount = double.tryParse(tx.totalAmount) ?? 0.0;
    final size = MediaQuery.of(context).size;

    return ClipRRect(
      borderRadius: const BorderRadius.vertical(top: Radius.circular(20)),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: size.height * 0.85),
          decoration: BoxDecoration(
            color: themeState.isDark 
              ? Color(themeState.slate950).withValues(alpha: 0.85) 
              : Color(themeState.bgHeader).withValues(alpha: 0.92),
            borderRadius: const BorderRadius.vertical(top: Radius.circular(20)),
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
                // Header
                Row(
                  children: [
                    Icon(LucideIcons.receipt, color: Color(themeState.accentColor), size: 24),
                    const SizedBox(width: 16),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text('TRANSACTION DETAILS', style: TextStyle(color: Color(themeState.accentColor), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 2)),
                          Text(tx.merchantName ?? 'Unknown Merchant', style: TextStyle(color: Color(themeState.textMain), fontSize: 20, fontWeight: FontWeight.bold)),
                        ],
                      ),
                    ),
                    IconButton(onPressed: () => Navigator.pop(context), icon: Icon(LucideIcons.x, color: Color(themeState.textMuted))),
                  ],
                ),
                const SizedBox(height: 32),

                // Amount & Date
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('TOTAL AMOUNT', style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.6), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
                        const SizedBox(height: 4),
                        Text(currencyFormat.format(amount), style: TextStyle(color: Color(themeState.accentColor), fontSize: 24, fontWeight: FontWeight.w900, fontFamily: 'monospace')),
                      ],
                    ),
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.end,
                      children: [
                        Text('DATE', style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.6), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
                        const SizedBox(height: 4),
                        Text(
                          DateFormat('MMM d, yyyy').format(DateTime.parse(tx.createdAt)),
                          style: TextStyle(color: Color(themeState.textMain), fontSize: 14, fontWeight: FontWeight.bold),
                        ),
                        Text(
                          DateFormat('HH:mm:ss').format(DateTime.parse(tx.createdAt)),
                          style: TextStyle(color: Color(themeState.textMuted), fontSize: 11, fontFamily: 'monospace'),
                        ),
                      ],
                    ),
                  ],
                ),
                const SizedBox(height: 32),

                // Category & Meta
                Row(
                  children: [
                    if (tx.category != null) ...[
                      _metaChip(themeState, LucideIcons.tag, tx.category!.toUpperCase()),
                      const SizedBox(width: 12),
                    ],
                    if (tx.userDisplayName != null)
                      _metaChip(themeState, LucideIcons.user, '@${tx.userDisplayName}'),
                  ],
                ),
                const SizedBox(height: 32),

                // Description
                if (tx.description != null && tx.description!.isNotEmpty) ...[
                  Text('DESCRIPTION', style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.6), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
                  const SizedBox(height: 12),
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(16),
                    decoration: BoxDecoration(
                      color: Color(themeState.textMain).withValues(alpha: 0.02),
                      borderRadius: BorderRadius.circular(16),
                      border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
                    ),
                    child: Text(tx.description!, style: TextStyle(color: Color(themeState.textMain), fontSize: 14, height: 1.6)),
                  ),
                  const SizedBox(height: 32),
                ],

                // Items List
                Text('PURCHASED ITEMS', style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.6), fontSize: 9, fontWeight: FontWeight.w900, letterSpacing: 1.5)),
                const SizedBox(height: 16),
                if (tx.items != null && tx.items!.isNotEmpty)
                  ...tx.items!.map((item) => _buildItemRow(themeState, item, currencyFormat))
                else
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.symmetric(vertical: 24),
                    decoration: BoxDecoration(
                      color: Color(themeState.textMain).withValues(alpha: 0.02),
                      borderRadius: BorderRadius.circular(16),
                      border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
                    ),
                    child: Center(
                      child: Text('No specific items recorded.', style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.5), fontSize: 12, fontStyle: FontStyle.italic)),
                    ),
                  ),
                const SizedBox(height: 40),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _metaChip(NomiTheme themeState, IconData icon, String label) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: Color(themeState.textMain).withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 12, color: Color(themeState.textMuted)),
          const SizedBox(width: 8),
          Text(label, style: TextStyle(color: Color(themeState.textMain), fontSize: 10, fontWeight: FontWeight.bold, letterSpacing: 1)),
        ],
      ),
    );
  }

  Widget _buildItemRow(NomiTheme themeState, TransactionItem item, NumberFormat currencyFormat) {
    final itemAmount = double.tryParse(item.totalAmount) ?? 0.0;
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Color(themeState.textMain).withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(item.name, style: TextStyle(color: Color(themeState.textMain), fontSize: 14, fontWeight: FontWeight.bold)),
                const SizedBox(height: 4),
                Text('QTY: ${item.quantity}', style: TextStyle(color: Color(themeState.textMuted), fontSize: 10, fontWeight: FontWeight.w900, letterSpacing: 1)),
              ],
            ),
          ),
          Text(
            currencyFormat.format(itemAmount),
            style: TextStyle(color: Color(themeState.textMain), fontSize: 14, fontWeight: FontWeight.bold, fontFamily: 'monospace'),
          ),
        ],
      ),
    );
  }
}

class _TransactionListItem extends ConsumerWidget {
  final Transaction tx;
  final VoidCallback onTap;
  const _TransactionListItem({required this.tx, required this.onTap});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeState = ref.watch(themeProvider);
    final currencyFormat = NumberFormat.currency(locale: 'id_ID', symbol: 'Rp ', decimalDigits: 0);
    final amount = double.tryParse(tx.totalAmount) ?? 0.0;
    
    return GestureDetector(
      onTap: onTap,
      child: Container(
        margin: const EdgeInsets.only(bottom: 12),
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: Color(themeState.textMain).withValues(alpha: 0.05),
          borderRadius: BorderRadius.circular(20),
          border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
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
                        style: TextStyle(color: Color(themeState.textMain), fontSize: 14, fontWeight: FontWeight.bold),
                      ),
                      if (tx.category != null) ...[
                        const SizedBox(width: 8),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: Color(themeState.textMain).withValues(alpha: 0.05),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            tx.category!.toUpperCase(),
                            style: TextStyle(color: Color(themeState.textMuted), fontSize: 7, fontWeight: FontWeight.w900, letterSpacing: 1),
                          ),
                        ),
                      ],
                    ],
                  ),
                  const SizedBox(height: 4),
                  Text(
                    tx.description ?? 'No description',
                    style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.8), fontSize: 12),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    DateFormat('MMM d, HH:mm').format(DateTime.parse(tx.createdAt)),
                    style: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.4), fontSize: 9, fontWeight: FontWeight.bold, fontFamily: 'monospace'),
                  ),
                ],
              ),
            ),
            Column(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                Text(
                  currencyFormat.format(amount),
                  style: TextStyle(color: Color(themeState.accentColor), fontSize: 14, fontWeight: FontWeight.w900, fontFamily: 'monospace'),
                ),
                const SizedBox(height: 4),
                if (tx.userDisplayName != null)
                  Text(
                    '@${tx.userDisplayName}',
                    style: TextStyle(color: Color(themeState.primaryColor), fontSize: 8, fontWeight: FontWeight.bold, fontStyle: FontStyle.italic),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

final transactionsStreamProvider = StreamProvider.family<List<db.Transaction>, (String?, String?)>((ref, params) {
  return ref.watch(chatRepositoryProvider).watchTransactions(category: params.$1, search: params.$2);
});
