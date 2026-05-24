import 'dart:io';
import 'package:drift/drift.dart';
import 'package:drift/native.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as p;

part 'database.g.dart';

// 📋 Table Definitions
class Conversations extends Table {
  TextColumn get id => text()();
  TextColumn get name => text().nullable()();
  IntColumn get cumulativeTokens => integer().nullable()();
  IntColumn get maxTokenUsage => integer().nullable()();
  DateTimeColumn get createdAt => dateTime()();
  DateTimeColumn get updatedAt => dateTime()();

  @override
  Set<Column> get primaryKey => {id};
}

class Messages extends Table {
  TextColumn get id => text()();
  TextColumn get conversationId => text().references(Conversations, #id)();
  TextColumn get role => text()(); // 'user', 'assistant', 'system'
  TextColumn get content => text()();
  TextColumn get displayName => text().nullable()();
  TextColumn get thought => text().nullable()();
  TextColumn get imageUrl => text().nullable()();
  TextColumn get videoUrl => text().nullable()();
  TextColumn get audioUrl => text().nullable()();
  TextColumn get documentUrl => text().nullable()();
  TextColumn get stickerUrl => text().nullable()();
  TextColumn get userId => text().nullable()();
  IntColumn get totalTokens => integer().nullable()();
  DateTimeColumn get createdAt => dateTime()();
  TextColumn get metadata => text().nullable()(); // JSON string
  TextColumn get replyToId => text().nullable()();
  TextColumn get repliedMessage => text().nullable()(); // JSON string

  // 🛰️ Offline-First Metadata
  IntColumn get syncStatus => intEnum<SyncStatus>()(); // 0: PENDING, 1: SYNCED, 2: FAILED

  @override
  Set<Column> get primaryKey => {id};
}

enum SyncStatus { pending, synced, failed }

@DriftDatabase(tables: [Conversations, Messages])
class NomiDatabase extends _$NomiDatabase {
  NomiDatabase() : super(_openConnection());

  @override
  int get schemaVersion => 1;

  // 📥 Sync: Conversations
  Future<void> upsertConversations(List<ConversationsCompanion> items) async {
    await batch((batch) {
      batch.insertAllOnConflictUpdate(conversations, items);
    });
  }

  Stream<List<Conversation>> watchConversations() {
    return (select(conversations)
          ..orderBy([
            (t) => OrderingTerm(expression: t.updatedAt, mode: OrderingMode.desc)
          ]))
        .watch();
  }

  // 📥 Sync: Messages
  Future<void> upsertMessages(List<MessagesCompanion> items) async {
    await batch((batch) {
      batch.insertAllOnConflictUpdate(messages, items);
    });
  }

  Stream<List<Message>> watchMessages(String conversationId) {
    return (select(messages)
          ..where((t) => t.conversationId.equals(conversationId))
          ..orderBy([
            (t) => OrderingTerm(expression: t.createdAt, mode: OrderingMode.desc)
          ]))
        .watch();
  }

  // 📤 Outbox: Get pending messages
  Future<List<Message>> getPendingMessages() {
    return (select(messages)..where((t) => t.syncStatus.equals(SyncStatus.pending.index))).get();
  }
}

LazyDatabase _openConnection() {
  return LazyDatabase(() async {
    final dbFolder = await getApplicationDocumentsDirectory();
    final file = File(p.join(dbFolder.path, 'nomi.sqlite'));
    return NativeDatabase(file);
  });
}
