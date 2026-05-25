// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'transaction.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Transaction _$TransactionFromJson(Map<String, dynamic> json) => Transaction(
  id: json['id'] as String,
  merchantName: json['merchant_name'] as String?,
  category: json['category'] as String?,
  description: json['description'] as String?,
  totalAmount: Transaction._parseAmount(json['total_amount']),
  createdAt: json['created_at'] as String,
  userDisplayName: json['user_display_name'] as String?,
  conversationTitle: json['conversation_title'] as String?,
  items:
      (json['items'] as List<dynamic>?)
          ?.map((e) => TransactionItem.fromJson(e as Map<String, dynamic>))
          .toList(),
);

Map<String, dynamic> _$TransactionToJson(Transaction instance) =>
    <String, dynamic>{
      'id': instance.id,
      'merchant_name': instance.merchantName,
      'category': instance.category,
      'description': instance.description,
      'total_amount': instance.totalAmount,
      'created_at': instance.createdAt,
      'user_display_name': instance.userDisplayName,
      'conversation_title': instance.conversationTitle,
      'items': instance.items,
    };

TransactionItem _$TransactionItemFromJson(Map<String, dynamic> json) =>
    TransactionItem(
      name: json['name'] as String,
      quantity: (json['quantity'] as num).toInt(),
      totalAmount: Transaction._parseAmount(json['total_amount']),
    );

Map<String, dynamic> _$TransactionItemToJson(TransactionItem instance) =>
    <String, dynamic>{
      'name': instance.name,
      'quantity': instance.quantity,
      'total_amount': instance.totalAmount,
    };
