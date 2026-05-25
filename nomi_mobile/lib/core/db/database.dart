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

class Reminders extends Table {
  TextColumn get id => text()();
  TextColumn get content => text()();
  TextColumn get taskType => text().nullable()(); // 'REMINDER', 'SEND_DM', 'TRIGGER_AGENT'
  TextColumn get frequency => text().nullable()(); // 'once', 'daily', etc.
  TextColumn get status => text()(); // 'pending', 'completed', 'archived'
  DateTimeColumn get dueAt => dateTime()();
  DateTimeColumn get createdAt => dateTime()();
  TextColumn get userDisplayName => text().nullable()();
  TextColumn get conversationTitle => text().nullable()();

  @override
  Set<Column> get primaryKey => {id};
}

class Transactions extends Table {
  TextColumn get id => text()();
  TextColumn get merchantName => text().nullable()();
  TextColumn get category => text().nullable()();
  TextColumn get description => text().nullable()();
  TextColumn get totalAmount => text()(); // String to maintain precision
  DateTimeColumn get createdAt => dateTime()();
  TextColumn get userDisplayName => text().nullable()();
  TextColumn get conversationTitle => text().nullable()();
  TextColumn get itemsJson => text().nullable()(); // JSON string for line items

  @override
  Set<Column> get primaryKey => {id};
}

class HealthMetrics extends Table {
  TextColumn get id => text()();
  TextColumn get userId => text()();
  DateTimeColumn get logDate => dateTime()();
  IntColumn get steps => integer().withDefault(const Constant(0))();
  IntColumn get avgHeartRate => integer().nullable()();
  RealColumn get sleepHours => real().nullable()();
  DateTimeColumn get updatedAt => dateTime()();

  @override
  Set<Column> get primaryKey => {id};
}

class Plugins extends Table {
  TextColumn get id => text()();
  TextColumn get name => text()();
  TextColumn get slug => text()();
  TextColumn get description => text().nullable()();
  TextColumn get scriptCode => text().nullable()();
  TextColumn get version => text().nullable()();
  TextColumn get author => text().nullable()();
  TextColumn get intentsJson => text().nullable()(); // JSON array
  DateTimeColumn get createdAt => dateTime()();
  DateTimeColumn get updatedAt => dateTime()();

  @override
  Set<Column> get primaryKey => {id};
}

@DriftDatabase(tables: [Conversations, Messages, Reminders, Transactions, HealthMetrics, Plugins])
class NomiDatabase extends _$NomiDatabase {
  NomiDatabase() : super(_openConnection());

  @override
  int get schemaVersion => 6; // Incremented for scriptCode

  @override
  MigrationStrategy get migration => MigrationStrategy(
        onUpgrade: (m, from, to) async {
          if (from < 2) await m.createTable(reminders);
          if (from < 3) await m.createTable(transactions);
          if (from < 4) await m.createTable(healthMetrics);
          if (from < 5) await m.createTable(plugins);
          if (from < 6) await m.addColumn(plugins, plugins.scriptCode);
        },
      );

  // 📥 Sync: Conversations
  Future<void> upsertConversations(List<ConversationsCompanion> items) async {
    await batch((batch) => batch.insertAllOnConflictUpdate(conversations, items));
  }
  Stream<List<Conversation>> watchConversations() => 
    (select(conversations)..orderBy([(t) => OrderingTerm(expression: t.updatedAt, mode: OrderingMode.desc)])).watch();

  // 📥 Sync: Messages
  Future<void> upsertMessages(List<MessagesCompanion> items) async {
    await batch((batch) => batch.insertAllOnConflictUpdate(messages, items));
  }
  Stream<List<Message>> watchMessages(String conversationId) => 
    (select(messages)..where((t) => t.conversationId.equals(conversationId))..orderBy([(t) => OrderingTerm(expression: t.createdAt, mode: OrderingMode.desc)])).watch();
  Future<List<Message>> getPendingMessages() => (select(messages)..where((t) => t.syncStatus.equals(SyncStatus.pending.index))).get();

  // 📥 Sync: Reminders
  Future<void> upsertReminders(List<RemindersCompanion> items) async {
    await batch((batch) => batch.insertAllOnConflictUpdate(reminders, items));
  }
  Stream<List<Reminder>> watchReminders() => 
    (select(reminders)..orderBy([(t) => OrderingTerm(expression: t.dueAt, mode: OrderingMode.desc)])).watch();
  Future<Reminder?> getReminderById(String id) => (select(reminders)..where((t) => t.id.equals(id))).getSingleOrNull();

  // 📥 Sync: Transactions
  Future<void> upsertTransactions(List<TransactionsCompanion> items) async {
    await batch((batch) => batch.insertAllOnConflictUpdate(transactions, items));
  }
  Stream<List<Transaction>> watchTransactions({String? category, String? search}) {
    var query = select(transactions);
    if (category?.isNotEmpty ?? false) query.where((t) => t.category.equals(category!));
    if (search?.isNotEmpty ?? false) query.where((t) => t.merchantName.like('%$search%') | t.description.like('%$search%'));
    return (query..orderBy([(t) => OrderingTerm(expression: t.createdAt, mode: OrderingMode.desc)])).watch();
  }
  Future<Transaction?> getTransactionById(String id) => (select(transactions)..where((t) => t.id.equals(id))).getSingleOrNull();

  // 📥 Sync: Health Metrics
  Future<void> upsertHealthMetrics(List<HealthMetricsCompanion> items) async {
    await batch((batch) => batch.insertAllOnConflictUpdate(healthMetrics, items));
  }
  Stream<List<HealthMetric>> watchHealthHistory({DateTime? start, DateTime? end}) {
    var query = select(healthMetrics);
    if (start != null) query.where((t) => t.logDate.isBiggerOrEqualValue(start));
    if (end != null) query.where((t) => t.logDate.isSmallerOrEqualValue(end));
    return (query..orderBy([(t) => OrderingTerm(expression: t.logDate, mode: OrderingMode.desc)])).watch();
  }

  // 📥 Sync: Plugins
  Future<void> upsertPlugins(List<PluginsCompanion> items) async {
    await batch((batch) => batch.insertAllOnConflictUpdate(plugins, items));
  }
  Stream<List<Plugin>> watchPlugins({String? search}) {
    var query = select(plugins);
    if (search?.isNotEmpty ?? false) query.where((t) => t.name.like('%$search%') | t.slug.like('%$search%'));
    return (query..orderBy([(t) => OrderingTerm(expression: t.name, mode: OrderingMode.asc)])).watch();
  }
}

LazyDatabase _openConnection() {
  return LazyDatabase(() async {
    final dbFolder = await getApplicationDocumentsDirectory();
    final file = File(p.join(dbFolder.path, 'nomi.sqlite'));
    return NativeDatabase(file);
  });
}
