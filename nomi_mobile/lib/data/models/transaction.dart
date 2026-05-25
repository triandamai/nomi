import 'dart:convert';
import 'package:json_annotation/json_annotation.dart';
import 'package:nomi_mobile/core/db/database.dart' as db;

part 'transaction.g.dart';

@JsonSerializable()
class Transaction {
  final String id;
  @JsonKey(name: 'merchant_name')
  final String? merchantName;
  final String? category;
  final String? description;
  @JsonKey(name: 'total_amount', fromJson: _parseAmount)
  final String totalAmount;
  @JsonKey(name: 'created_at')
  final String createdAt;
  @JsonKey(name: 'user_display_name')
  final String? userDisplayName;
  @JsonKey(name: 'conversation_title')
  final String? conversationTitle;
  final List<TransactionItem>? items;

  Transaction({
    required this.id,
    this.merchantName,
    this.category,
    this.description,
    required this.totalAmount,
    required this.createdAt,
    this.userDisplayName,
    this.conversationTitle,
    this.items,
  });

  static String _parseAmount(dynamic value) {
    if (value == null) return '0';
    if (value is String) return value;
    return value.toString();
  }

  factory Transaction.fromJson(Map<String, dynamic> json) => _$TransactionFromJson(json);
  Map<String, dynamic> toJson() => _$TransactionToJson(this);

  // 🏛️ Bridge: From Drift to UI Model
  factory Transaction.fromDb(db.Transaction m) {
    List<TransactionItem>? itemsList;
    if (m.itemsJson != null) {
      try {
        final List<dynamic> decoded = jsonDecode(m.itemsJson!);
        itemsList = decoded.map((e) => TransactionItem.fromJson(e)).toList();
      } catch (_) {}
    }

    return Transaction(
      id: m.id,
      merchantName: m.merchantName,
      category: m.category,
      description: m.description,
      totalAmount: m.totalAmount,
      createdAt: m.createdAt.toIso8601String(),
      userDisplayName: m.userDisplayName,
      conversationTitle: m.conversationTitle,
      items: itemsList,
    );
  }
}

@JsonSerializable()
class TransactionItem {
  final String name;
  final int quantity;
  @JsonKey(name: 'total_amount', fromJson: Transaction._parseAmount)
  final String totalAmount;

  TransactionItem({
    required this.name,
    required this.quantity,
    required this.totalAmount,
  });

  factory TransactionItem.fromJson(Map<String, dynamic> json) => _$TransactionItemFromJson(json);
  Map<String, dynamic> toJson() => _$TransactionItemToJson(this);
}
