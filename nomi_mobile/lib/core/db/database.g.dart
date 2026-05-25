// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'database.dart';

// ignore_for_file: type=lint
class $ConversationsTable extends Conversations
    with TableInfo<$ConversationsTable, Conversation> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $ConversationsTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<String> id = GeneratedColumn<String>(
    'id',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _nameMeta = const VerificationMeta('name');
  @override
  late final GeneratedColumn<String> name = GeneratedColumn<String>(
    'name',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _cumulativeTokensMeta = const VerificationMeta(
    'cumulativeTokens',
  );
  @override
  late final GeneratedColumn<int> cumulativeTokens = GeneratedColumn<int>(
    'cumulative_tokens',
    aliasedName,
    true,
    type: DriftSqlType.int,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _maxTokenUsageMeta = const VerificationMeta(
    'maxTokenUsage',
  );
  @override
  late final GeneratedColumn<int> maxTokenUsage = GeneratedColumn<int>(
    'max_token_usage',
    aliasedName,
    true,
    type: DriftSqlType.int,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _createdAtMeta = const VerificationMeta(
    'createdAt',
  );
  @override
  late final GeneratedColumn<DateTime> createdAt = GeneratedColumn<DateTime>(
    'created_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _updatedAtMeta = const VerificationMeta(
    'updatedAt',
  );
  @override
  late final GeneratedColumn<DateTime> updatedAt = GeneratedColumn<DateTime>(
    'updated_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  @override
  List<GeneratedColumn> get $columns => [
    id,
    name,
    cumulativeTokens,
    maxTokenUsage,
    createdAt,
    updatedAt,
  ];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'conversations';
  @override
  VerificationContext validateIntegrity(
    Insertable<Conversation> instance, {
    bool isInserting = false,
  }) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('name')) {
      context.handle(
        _nameMeta,
        name.isAcceptableOrUnknown(data['name']!, _nameMeta),
      );
    }
    if (data.containsKey('cumulative_tokens')) {
      context.handle(
        _cumulativeTokensMeta,
        cumulativeTokens.isAcceptableOrUnknown(
          data['cumulative_tokens']!,
          _cumulativeTokensMeta,
        ),
      );
    }
    if (data.containsKey('max_token_usage')) {
      context.handle(
        _maxTokenUsageMeta,
        maxTokenUsage.isAcceptableOrUnknown(
          data['max_token_usage']!,
          _maxTokenUsageMeta,
        ),
      );
    }
    if (data.containsKey('created_at')) {
      context.handle(
        _createdAtMeta,
        createdAt.isAcceptableOrUnknown(data['created_at']!, _createdAtMeta),
      );
    } else if (isInserting) {
      context.missing(_createdAtMeta);
    }
    if (data.containsKey('updated_at')) {
      context.handle(
        _updatedAtMeta,
        updatedAt.isAcceptableOrUnknown(data['updated_at']!, _updatedAtMeta),
      );
    } else if (isInserting) {
      context.missing(_updatedAtMeta);
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  Conversation map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return Conversation(
      id:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}id'],
          )!,
      name: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}name'],
      ),
      cumulativeTokens: attachedDatabase.typeMapping.read(
        DriftSqlType.int,
        data['${effectivePrefix}cumulative_tokens'],
      ),
      maxTokenUsage: attachedDatabase.typeMapping.read(
        DriftSqlType.int,
        data['${effectivePrefix}max_token_usage'],
      ),
      createdAt:
          attachedDatabase.typeMapping.read(
            DriftSqlType.dateTime,
            data['${effectivePrefix}created_at'],
          )!,
      updatedAt:
          attachedDatabase.typeMapping.read(
            DriftSqlType.dateTime,
            data['${effectivePrefix}updated_at'],
          )!,
    );
  }

  @override
  $ConversationsTable createAlias(String alias) {
    return $ConversationsTable(attachedDatabase, alias);
  }
}

class Conversation extends DataClass implements Insertable<Conversation> {
  final String id;
  final String? name;
  final int? cumulativeTokens;
  final int? maxTokenUsage;
  final DateTime createdAt;
  final DateTime updatedAt;
  const Conversation({
    required this.id,
    this.name,
    this.cumulativeTokens,
    this.maxTokenUsage,
    required this.createdAt,
    required this.updatedAt,
  });
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    if (!nullToAbsent || name != null) {
      map['name'] = Variable<String>(name);
    }
    if (!nullToAbsent || cumulativeTokens != null) {
      map['cumulative_tokens'] = Variable<int>(cumulativeTokens);
    }
    if (!nullToAbsent || maxTokenUsage != null) {
      map['max_token_usage'] = Variable<int>(maxTokenUsage);
    }
    map['created_at'] = Variable<DateTime>(createdAt);
    map['updated_at'] = Variable<DateTime>(updatedAt);
    return map;
  }

  ConversationsCompanion toCompanion(bool nullToAbsent) {
    return ConversationsCompanion(
      id: Value(id),
      name: name == null && nullToAbsent ? const Value.absent() : Value(name),
      cumulativeTokens:
          cumulativeTokens == null && nullToAbsent
              ? const Value.absent()
              : Value(cumulativeTokens),
      maxTokenUsage:
          maxTokenUsage == null && nullToAbsent
              ? const Value.absent()
              : Value(maxTokenUsage),
      createdAt: Value(createdAt),
      updatedAt: Value(updatedAt),
    );
  }

  factory Conversation.fromJson(
    Map<String, dynamic> json, {
    ValueSerializer? serializer,
  }) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return Conversation(
      id: serializer.fromJson<String>(json['id']),
      name: serializer.fromJson<String?>(json['name']),
      cumulativeTokens: serializer.fromJson<int?>(json['cumulativeTokens']),
      maxTokenUsage: serializer.fromJson<int?>(json['maxTokenUsage']),
      createdAt: serializer.fromJson<DateTime>(json['createdAt']),
      updatedAt: serializer.fromJson<DateTime>(json['updatedAt']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<String>(id),
      'name': serializer.toJson<String?>(name),
      'cumulativeTokens': serializer.toJson<int?>(cumulativeTokens),
      'maxTokenUsage': serializer.toJson<int?>(maxTokenUsage),
      'createdAt': serializer.toJson<DateTime>(createdAt),
      'updatedAt': serializer.toJson<DateTime>(updatedAt),
    };
  }

  Conversation copyWith({
    String? id,
    Value<String?> name = const Value.absent(),
    Value<int?> cumulativeTokens = const Value.absent(),
    Value<int?> maxTokenUsage = const Value.absent(),
    DateTime? createdAt,
    DateTime? updatedAt,
  }) => Conversation(
    id: id ?? this.id,
    name: name.present ? name.value : this.name,
    cumulativeTokens:
        cumulativeTokens.present
            ? cumulativeTokens.value
            : this.cumulativeTokens,
    maxTokenUsage:
        maxTokenUsage.present ? maxTokenUsage.value : this.maxTokenUsage,
    createdAt: createdAt ?? this.createdAt,
    updatedAt: updatedAt ?? this.updatedAt,
  );
  Conversation copyWithCompanion(ConversationsCompanion data) {
    return Conversation(
      id: data.id.present ? data.id.value : this.id,
      name: data.name.present ? data.name.value : this.name,
      cumulativeTokens:
          data.cumulativeTokens.present
              ? data.cumulativeTokens.value
              : this.cumulativeTokens,
      maxTokenUsage:
          data.maxTokenUsage.present
              ? data.maxTokenUsage.value
              : this.maxTokenUsage,
      createdAt: data.createdAt.present ? data.createdAt.value : this.createdAt,
      updatedAt: data.updatedAt.present ? data.updatedAt.value : this.updatedAt,
    );
  }

  @override
  String toString() {
    return (StringBuffer('Conversation(')
          ..write('id: $id, ')
          ..write('name: $name, ')
          ..write('cumulativeTokens: $cumulativeTokens, ')
          ..write('maxTokenUsage: $maxTokenUsage, ')
          ..write('createdAt: $createdAt, ')
          ..write('updatedAt: $updatedAt')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(
    id,
    name,
    cumulativeTokens,
    maxTokenUsage,
    createdAt,
    updatedAt,
  );
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Conversation &&
          other.id == this.id &&
          other.name == this.name &&
          other.cumulativeTokens == this.cumulativeTokens &&
          other.maxTokenUsage == this.maxTokenUsage &&
          other.createdAt == this.createdAt &&
          other.updatedAt == this.updatedAt);
}

class ConversationsCompanion extends UpdateCompanion<Conversation> {
  final Value<String> id;
  final Value<String?> name;
  final Value<int?> cumulativeTokens;
  final Value<int?> maxTokenUsage;
  final Value<DateTime> createdAt;
  final Value<DateTime> updatedAt;
  final Value<int> rowid;
  const ConversationsCompanion({
    this.id = const Value.absent(),
    this.name = const Value.absent(),
    this.cumulativeTokens = const Value.absent(),
    this.maxTokenUsage = const Value.absent(),
    this.createdAt = const Value.absent(),
    this.updatedAt = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  ConversationsCompanion.insert({
    required String id,
    this.name = const Value.absent(),
    this.cumulativeTokens = const Value.absent(),
    this.maxTokenUsage = const Value.absent(),
    required DateTime createdAt,
    required DateTime updatedAt,
    this.rowid = const Value.absent(),
  }) : id = Value(id),
       createdAt = Value(createdAt),
       updatedAt = Value(updatedAt);
  static Insertable<Conversation> custom({
    Expression<String>? id,
    Expression<String>? name,
    Expression<int>? cumulativeTokens,
    Expression<int>? maxTokenUsage,
    Expression<DateTime>? createdAt,
    Expression<DateTime>? updatedAt,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (name != null) 'name': name,
      if (cumulativeTokens != null) 'cumulative_tokens': cumulativeTokens,
      if (maxTokenUsage != null) 'max_token_usage': maxTokenUsage,
      if (createdAt != null) 'created_at': createdAt,
      if (updatedAt != null) 'updated_at': updatedAt,
      if (rowid != null) 'rowid': rowid,
    });
  }

  ConversationsCompanion copyWith({
    Value<String>? id,
    Value<String?>? name,
    Value<int?>? cumulativeTokens,
    Value<int?>? maxTokenUsage,
    Value<DateTime>? createdAt,
    Value<DateTime>? updatedAt,
    Value<int>? rowid,
  }) {
    return ConversationsCompanion(
      id: id ?? this.id,
      name: name ?? this.name,
      cumulativeTokens: cumulativeTokens ?? this.cumulativeTokens,
      maxTokenUsage: maxTokenUsage ?? this.maxTokenUsage,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<String>(id.value);
    }
    if (name.present) {
      map['name'] = Variable<String>(name.value);
    }
    if (cumulativeTokens.present) {
      map['cumulative_tokens'] = Variable<int>(cumulativeTokens.value);
    }
    if (maxTokenUsage.present) {
      map['max_token_usage'] = Variable<int>(maxTokenUsage.value);
    }
    if (createdAt.present) {
      map['created_at'] = Variable<DateTime>(createdAt.value);
    }
    if (updatedAt.present) {
      map['updated_at'] = Variable<DateTime>(updatedAt.value);
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('ConversationsCompanion(')
          ..write('id: $id, ')
          ..write('name: $name, ')
          ..write('cumulativeTokens: $cumulativeTokens, ')
          ..write('maxTokenUsage: $maxTokenUsage, ')
          ..write('createdAt: $createdAt, ')
          ..write('updatedAt: $updatedAt, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

class $MessagesTable extends Messages with TableInfo<$MessagesTable, Message> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $MessagesTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<String> id = GeneratedColumn<String>(
    'id',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _conversationIdMeta = const VerificationMeta(
    'conversationId',
  );
  @override
  late final GeneratedColumn<String> conversationId = GeneratedColumn<String>(
    'conversation_id',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
    defaultConstraints: GeneratedColumn.constraintIsAlways(
      'REFERENCES conversations (id)',
    ),
  );
  static const VerificationMeta _roleMeta = const VerificationMeta('role');
  @override
  late final GeneratedColumn<String> role = GeneratedColumn<String>(
    'role',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _contentMeta = const VerificationMeta(
    'content',
  );
  @override
  late final GeneratedColumn<String> content = GeneratedColumn<String>(
    'content',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _displayNameMeta = const VerificationMeta(
    'displayName',
  );
  @override
  late final GeneratedColumn<String> displayName = GeneratedColumn<String>(
    'display_name',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _thoughtMeta = const VerificationMeta(
    'thought',
  );
  @override
  late final GeneratedColumn<String> thought = GeneratedColumn<String>(
    'thought',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _imageUrlMeta = const VerificationMeta(
    'imageUrl',
  );
  @override
  late final GeneratedColumn<String> imageUrl = GeneratedColumn<String>(
    'image_url',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _videoUrlMeta = const VerificationMeta(
    'videoUrl',
  );
  @override
  late final GeneratedColumn<String> videoUrl = GeneratedColumn<String>(
    'video_url',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _audioUrlMeta = const VerificationMeta(
    'audioUrl',
  );
  @override
  late final GeneratedColumn<String> audioUrl = GeneratedColumn<String>(
    'audio_url',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _documentUrlMeta = const VerificationMeta(
    'documentUrl',
  );
  @override
  late final GeneratedColumn<String> documentUrl = GeneratedColumn<String>(
    'document_url',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _stickerUrlMeta = const VerificationMeta(
    'stickerUrl',
  );
  @override
  late final GeneratedColumn<String> stickerUrl = GeneratedColumn<String>(
    'sticker_url',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _userIdMeta = const VerificationMeta('userId');
  @override
  late final GeneratedColumn<String> userId = GeneratedColumn<String>(
    'user_id',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _totalTokensMeta = const VerificationMeta(
    'totalTokens',
  );
  @override
  late final GeneratedColumn<int> totalTokens = GeneratedColumn<int>(
    'total_tokens',
    aliasedName,
    true,
    type: DriftSqlType.int,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _createdAtMeta = const VerificationMeta(
    'createdAt',
  );
  @override
  late final GeneratedColumn<DateTime> createdAt = GeneratedColumn<DateTime>(
    'created_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _metadataMeta = const VerificationMeta(
    'metadata',
  );
  @override
  late final GeneratedColumn<String> metadata = GeneratedColumn<String>(
    'metadata',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _replyToIdMeta = const VerificationMeta(
    'replyToId',
  );
  @override
  late final GeneratedColumn<String> replyToId = GeneratedColumn<String>(
    'reply_to_id',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _repliedMessageMeta = const VerificationMeta(
    'repliedMessage',
  );
  @override
  late final GeneratedColumn<String> repliedMessage = GeneratedColumn<String>(
    'replied_message',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  @override
  late final GeneratedColumnWithTypeConverter<SyncStatus, int> syncStatus =
      GeneratedColumn<int>(
        'sync_status',
        aliasedName,
        false,
        type: DriftSqlType.int,
        requiredDuringInsert: true,
      ).withConverter<SyncStatus>($MessagesTable.$convertersyncStatus);
  @override
  List<GeneratedColumn> get $columns => [
    id,
    conversationId,
    role,
    content,
    displayName,
    thought,
    imageUrl,
    videoUrl,
    audioUrl,
    documentUrl,
    stickerUrl,
    userId,
    totalTokens,
    createdAt,
    metadata,
    replyToId,
    repliedMessage,
    syncStatus,
  ];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'messages';
  @override
  VerificationContext validateIntegrity(
    Insertable<Message> instance, {
    bool isInserting = false,
  }) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('conversation_id')) {
      context.handle(
        _conversationIdMeta,
        conversationId.isAcceptableOrUnknown(
          data['conversation_id']!,
          _conversationIdMeta,
        ),
      );
    } else if (isInserting) {
      context.missing(_conversationIdMeta);
    }
    if (data.containsKey('role')) {
      context.handle(
        _roleMeta,
        role.isAcceptableOrUnknown(data['role']!, _roleMeta),
      );
    } else if (isInserting) {
      context.missing(_roleMeta);
    }
    if (data.containsKey('content')) {
      context.handle(
        _contentMeta,
        content.isAcceptableOrUnknown(data['content']!, _contentMeta),
      );
    } else if (isInserting) {
      context.missing(_contentMeta);
    }
    if (data.containsKey('display_name')) {
      context.handle(
        _displayNameMeta,
        displayName.isAcceptableOrUnknown(
          data['display_name']!,
          _displayNameMeta,
        ),
      );
    }
    if (data.containsKey('thought')) {
      context.handle(
        _thoughtMeta,
        thought.isAcceptableOrUnknown(data['thought']!, _thoughtMeta),
      );
    }
    if (data.containsKey('image_url')) {
      context.handle(
        _imageUrlMeta,
        imageUrl.isAcceptableOrUnknown(data['image_url']!, _imageUrlMeta),
      );
    }
    if (data.containsKey('video_url')) {
      context.handle(
        _videoUrlMeta,
        videoUrl.isAcceptableOrUnknown(data['video_url']!, _videoUrlMeta),
      );
    }
    if (data.containsKey('audio_url')) {
      context.handle(
        _audioUrlMeta,
        audioUrl.isAcceptableOrUnknown(data['audio_url']!, _audioUrlMeta),
      );
    }
    if (data.containsKey('document_url')) {
      context.handle(
        _documentUrlMeta,
        documentUrl.isAcceptableOrUnknown(
          data['document_url']!,
          _documentUrlMeta,
        ),
      );
    }
    if (data.containsKey('sticker_url')) {
      context.handle(
        _stickerUrlMeta,
        stickerUrl.isAcceptableOrUnknown(data['sticker_url']!, _stickerUrlMeta),
      );
    }
    if (data.containsKey('user_id')) {
      context.handle(
        _userIdMeta,
        userId.isAcceptableOrUnknown(data['user_id']!, _userIdMeta),
      );
    }
    if (data.containsKey('total_tokens')) {
      context.handle(
        _totalTokensMeta,
        totalTokens.isAcceptableOrUnknown(
          data['total_tokens']!,
          _totalTokensMeta,
        ),
      );
    }
    if (data.containsKey('created_at')) {
      context.handle(
        _createdAtMeta,
        createdAt.isAcceptableOrUnknown(data['created_at']!, _createdAtMeta),
      );
    } else if (isInserting) {
      context.missing(_createdAtMeta);
    }
    if (data.containsKey('metadata')) {
      context.handle(
        _metadataMeta,
        metadata.isAcceptableOrUnknown(data['metadata']!, _metadataMeta),
      );
    }
    if (data.containsKey('reply_to_id')) {
      context.handle(
        _replyToIdMeta,
        replyToId.isAcceptableOrUnknown(data['reply_to_id']!, _replyToIdMeta),
      );
    }
    if (data.containsKey('replied_message')) {
      context.handle(
        _repliedMessageMeta,
        repliedMessage.isAcceptableOrUnknown(
          data['replied_message']!,
          _repliedMessageMeta,
        ),
      );
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  Message map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return Message(
      id:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}id'],
          )!,
      conversationId:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}conversation_id'],
          )!,
      role:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}role'],
          )!,
      content:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}content'],
          )!,
      displayName: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}display_name'],
      ),
      thought: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}thought'],
      ),
      imageUrl: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}image_url'],
      ),
      videoUrl: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}video_url'],
      ),
      audioUrl: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}audio_url'],
      ),
      documentUrl: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}document_url'],
      ),
      stickerUrl: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}sticker_url'],
      ),
      userId: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}user_id'],
      ),
      totalTokens: attachedDatabase.typeMapping.read(
        DriftSqlType.int,
        data['${effectivePrefix}total_tokens'],
      ),
      createdAt:
          attachedDatabase.typeMapping.read(
            DriftSqlType.dateTime,
            data['${effectivePrefix}created_at'],
          )!,
      metadata: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}metadata'],
      ),
      replyToId: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}reply_to_id'],
      ),
      repliedMessage: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}replied_message'],
      ),
      syncStatus: $MessagesTable.$convertersyncStatus.fromSql(
        attachedDatabase.typeMapping.read(
          DriftSqlType.int,
          data['${effectivePrefix}sync_status'],
        )!,
      ),
    );
  }

  @override
  $MessagesTable createAlias(String alias) {
    return $MessagesTable(attachedDatabase, alias);
  }

  static JsonTypeConverter2<SyncStatus, int, int> $convertersyncStatus =
      const EnumIndexConverter<SyncStatus>(SyncStatus.values);
}

class Message extends DataClass implements Insertable<Message> {
  final String id;
  final String conversationId;
  final String role;
  final String content;
  final String? displayName;
  final String? thought;
  final String? imageUrl;
  final String? videoUrl;
  final String? audioUrl;
  final String? documentUrl;
  final String? stickerUrl;
  final String? userId;
  final int? totalTokens;
  final DateTime createdAt;
  final String? metadata;
  final String? replyToId;
  final String? repliedMessage;
  final SyncStatus syncStatus;
  const Message({
    required this.id,
    required this.conversationId,
    required this.role,
    required this.content,
    this.displayName,
    this.thought,
    this.imageUrl,
    this.videoUrl,
    this.audioUrl,
    this.documentUrl,
    this.stickerUrl,
    this.userId,
    this.totalTokens,
    required this.createdAt,
    this.metadata,
    this.replyToId,
    this.repliedMessage,
    required this.syncStatus,
  });
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    map['conversation_id'] = Variable<String>(conversationId);
    map['role'] = Variable<String>(role);
    map['content'] = Variable<String>(content);
    if (!nullToAbsent || displayName != null) {
      map['display_name'] = Variable<String>(displayName);
    }
    if (!nullToAbsent || thought != null) {
      map['thought'] = Variable<String>(thought);
    }
    if (!nullToAbsent || imageUrl != null) {
      map['image_url'] = Variable<String>(imageUrl);
    }
    if (!nullToAbsent || videoUrl != null) {
      map['video_url'] = Variable<String>(videoUrl);
    }
    if (!nullToAbsent || audioUrl != null) {
      map['audio_url'] = Variable<String>(audioUrl);
    }
    if (!nullToAbsent || documentUrl != null) {
      map['document_url'] = Variable<String>(documentUrl);
    }
    if (!nullToAbsent || stickerUrl != null) {
      map['sticker_url'] = Variable<String>(stickerUrl);
    }
    if (!nullToAbsent || userId != null) {
      map['user_id'] = Variable<String>(userId);
    }
    if (!nullToAbsent || totalTokens != null) {
      map['total_tokens'] = Variable<int>(totalTokens);
    }
    map['created_at'] = Variable<DateTime>(createdAt);
    if (!nullToAbsent || metadata != null) {
      map['metadata'] = Variable<String>(metadata);
    }
    if (!nullToAbsent || replyToId != null) {
      map['reply_to_id'] = Variable<String>(replyToId);
    }
    if (!nullToAbsent || repliedMessage != null) {
      map['replied_message'] = Variable<String>(repliedMessage);
    }
    {
      map['sync_status'] = Variable<int>(
        $MessagesTable.$convertersyncStatus.toSql(syncStatus),
      );
    }
    return map;
  }

  MessagesCompanion toCompanion(bool nullToAbsent) {
    return MessagesCompanion(
      id: Value(id),
      conversationId: Value(conversationId),
      role: Value(role),
      content: Value(content),
      displayName:
          displayName == null && nullToAbsent
              ? const Value.absent()
              : Value(displayName),
      thought:
          thought == null && nullToAbsent
              ? const Value.absent()
              : Value(thought),
      imageUrl:
          imageUrl == null && nullToAbsent
              ? const Value.absent()
              : Value(imageUrl),
      videoUrl:
          videoUrl == null && nullToAbsent
              ? const Value.absent()
              : Value(videoUrl),
      audioUrl:
          audioUrl == null && nullToAbsent
              ? const Value.absent()
              : Value(audioUrl),
      documentUrl:
          documentUrl == null && nullToAbsent
              ? const Value.absent()
              : Value(documentUrl),
      stickerUrl:
          stickerUrl == null && nullToAbsent
              ? const Value.absent()
              : Value(stickerUrl),
      userId:
          userId == null && nullToAbsent ? const Value.absent() : Value(userId),
      totalTokens:
          totalTokens == null && nullToAbsent
              ? const Value.absent()
              : Value(totalTokens),
      createdAt: Value(createdAt),
      metadata:
          metadata == null && nullToAbsent
              ? const Value.absent()
              : Value(metadata),
      replyToId:
          replyToId == null && nullToAbsent
              ? const Value.absent()
              : Value(replyToId),
      repliedMessage:
          repliedMessage == null && nullToAbsent
              ? const Value.absent()
              : Value(repliedMessage),
      syncStatus: Value(syncStatus),
    );
  }

  factory Message.fromJson(
    Map<String, dynamic> json, {
    ValueSerializer? serializer,
  }) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return Message(
      id: serializer.fromJson<String>(json['id']),
      conversationId: serializer.fromJson<String>(json['conversationId']),
      role: serializer.fromJson<String>(json['role']),
      content: serializer.fromJson<String>(json['content']),
      displayName: serializer.fromJson<String?>(json['displayName']),
      thought: serializer.fromJson<String?>(json['thought']),
      imageUrl: serializer.fromJson<String?>(json['imageUrl']),
      videoUrl: serializer.fromJson<String?>(json['videoUrl']),
      audioUrl: serializer.fromJson<String?>(json['audioUrl']),
      documentUrl: serializer.fromJson<String?>(json['documentUrl']),
      stickerUrl: serializer.fromJson<String?>(json['stickerUrl']),
      userId: serializer.fromJson<String?>(json['userId']),
      totalTokens: serializer.fromJson<int?>(json['totalTokens']),
      createdAt: serializer.fromJson<DateTime>(json['createdAt']),
      metadata: serializer.fromJson<String?>(json['metadata']),
      replyToId: serializer.fromJson<String?>(json['replyToId']),
      repliedMessage: serializer.fromJson<String?>(json['repliedMessage']),
      syncStatus: $MessagesTable.$convertersyncStatus.fromJson(
        serializer.fromJson<int>(json['syncStatus']),
      ),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<String>(id),
      'conversationId': serializer.toJson<String>(conversationId),
      'role': serializer.toJson<String>(role),
      'content': serializer.toJson<String>(content),
      'displayName': serializer.toJson<String?>(displayName),
      'thought': serializer.toJson<String?>(thought),
      'imageUrl': serializer.toJson<String?>(imageUrl),
      'videoUrl': serializer.toJson<String?>(videoUrl),
      'audioUrl': serializer.toJson<String?>(audioUrl),
      'documentUrl': serializer.toJson<String?>(documentUrl),
      'stickerUrl': serializer.toJson<String?>(stickerUrl),
      'userId': serializer.toJson<String?>(userId),
      'totalTokens': serializer.toJson<int?>(totalTokens),
      'createdAt': serializer.toJson<DateTime>(createdAt),
      'metadata': serializer.toJson<String?>(metadata),
      'replyToId': serializer.toJson<String?>(replyToId),
      'repliedMessage': serializer.toJson<String?>(repliedMessage),
      'syncStatus': serializer.toJson<int>(
        $MessagesTable.$convertersyncStatus.toJson(syncStatus),
      ),
    };
  }

  Message copyWith({
    String? id,
    String? conversationId,
    String? role,
    String? content,
    Value<String?> displayName = const Value.absent(),
    Value<String?> thought = const Value.absent(),
    Value<String?> imageUrl = const Value.absent(),
    Value<String?> videoUrl = const Value.absent(),
    Value<String?> audioUrl = const Value.absent(),
    Value<String?> documentUrl = const Value.absent(),
    Value<String?> stickerUrl = const Value.absent(),
    Value<String?> userId = const Value.absent(),
    Value<int?> totalTokens = const Value.absent(),
    DateTime? createdAt,
    Value<String?> metadata = const Value.absent(),
    Value<String?> replyToId = const Value.absent(),
    Value<String?> repliedMessage = const Value.absent(),
    SyncStatus? syncStatus,
  }) => Message(
    id: id ?? this.id,
    conversationId: conversationId ?? this.conversationId,
    role: role ?? this.role,
    content: content ?? this.content,
    displayName: displayName.present ? displayName.value : this.displayName,
    thought: thought.present ? thought.value : this.thought,
    imageUrl: imageUrl.present ? imageUrl.value : this.imageUrl,
    videoUrl: videoUrl.present ? videoUrl.value : this.videoUrl,
    audioUrl: audioUrl.present ? audioUrl.value : this.audioUrl,
    documentUrl: documentUrl.present ? documentUrl.value : this.documentUrl,
    stickerUrl: stickerUrl.present ? stickerUrl.value : this.stickerUrl,
    userId: userId.present ? userId.value : this.userId,
    totalTokens: totalTokens.present ? totalTokens.value : this.totalTokens,
    createdAt: createdAt ?? this.createdAt,
    metadata: metadata.present ? metadata.value : this.metadata,
    replyToId: replyToId.present ? replyToId.value : this.replyToId,
    repliedMessage:
        repliedMessage.present ? repliedMessage.value : this.repliedMessage,
    syncStatus: syncStatus ?? this.syncStatus,
  );
  Message copyWithCompanion(MessagesCompanion data) {
    return Message(
      id: data.id.present ? data.id.value : this.id,
      conversationId:
          data.conversationId.present
              ? data.conversationId.value
              : this.conversationId,
      role: data.role.present ? data.role.value : this.role,
      content: data.content.present ? data.content.value : this.content,
      displayName:
          data.displayName.present ? data.displayName.value : this.displayName,
      thought: data.thought.present ? data.thought.value : this.thought,
      imageUrl: data.imageUrl.present ? data.imageUrl.value : this.imageUrl,
      videoUrl: data.videoUrl.present ? data.videoUrl.value : this.videoUrl,
      audioUrl: data.audioUrl.present ? data.audioUrl.value : this.audioUrl,
      documentUrl:
          data.documentUrl.present ? data.documentUrl.value : this.documentUrl,
      stickerUrl:
          data.stickerUrl.present ? data.stickerUrl.value : this.stickerUrl,
      userId: data.userId.present ? data.userId.value : this.userId,
      totalTokens:
          data.totalTokens.present ? data.totalTokens.value : this.totalTokens,
      createdAt: data.createdAt.present ? data.createdAt.value : this.createdAt,
      metadata: data.metadata.present ? data.metadata.value : this.metadata,
      replyToId: data.replyToId.present ? data.replyToId.value : this.replyToId,
      repliedMessage:
          data.repliedMessage.present
              ? data.repliedMessage.value
              : this.repliedMessage,
      syncStatus:
          data.syncStatus.present ? data.syncStatus.value : this.syncStatus,
    );
  }

  @override
  String toString() {
    return (StringBuffer('Message(')
          ..write('id: $id, ')
          ..write('conversationId: $conversationId, ')
          ..write('role: $role, ')
          ..write('content: $content, ')
          ..write('displayName: $displayName, ')
          ..write('thought: $thought, ')
          ..write('imageUrl: $imageUrl, ')
          ..write('videoUrl: $videoUrl, ')
          ..write('audioUrl: $audioUrl, ')
          ..write('documentUrl: $documentUrl, ')
          ..write('stickerUrl: $stickerUrl, ')
          ..write('userId: $userId, ')
          ..write('totalTokens: $totalTokens, ')
          ..write('createdAt: $createdAt, ')
          ..write('metadata: $metadata, ')
          ..write('replyToId: $replyToId, ')
          ..write('repliedMessage: $repliedMessage, ')
          ..write('syncStatus: $syncStatus')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(
    id,
    conversationId,
    role,
    content,
    displayName,
    thought,
    imageUrl,
    videoUrl,
    audioUrl,
    documentUrl,
    stickerUrl,
    userId,
    totalTokens,
    createdAt,
    metadata,
    replyToId,
    repliedMessage,
    syncStatus,
  );
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Message &&
          other.id == this.id &&
          other.conversationId == this.conversationId &&
          other.role == this.role &&
          other.content == this.content &&
          other.displayName == this.displayName &&
          other.thought == this.thought &&
          other.imageUrl == this.imageUrl &&
          other.videoUrl == this.videoUrl &&
          other.audioUrl == this.audioUrl &&
          other.documentUrl == this.documentUrl &&
          other.stickerUrl == this.stickerUrl &&
          other.userId == this.userId &&
          other.totalTokens == this.totalTokens &&
          other.createdAt == this.createdAt &&
          other.metadata == this.metadata &&
          other.replyToId == this.replyToId &&
          other.repliedMessage == this.repliedMessage &&
          other.syncStatus == this.syncStatus);
}

class MessagesCompanion extends UpdateCompanion<Message> {
  final Value<String> id;
  final Value<String> conversationId;
  final Value<String> role;
  final Value<String> content;
  final Value<String?> displayName;
  final Value<String?> thought;
  final Value<String?> imageUrl;
  final Value<String?> videoUrl;
  final Value<String?> audioUrl;
  final Value<String?> documentUrl;
  final Value<String?> stickerUrl;
  final Value<String?> userId;
  final Value<int?> totalTokens;
  final Value<DateTime> createdAt;
  final Value<String?> metadata;
  final Value<String?> replyToId;
  final Value<String?> repliedMessage;
  final Value<SyncStatus> syncStatus;
  final Value<int> rowid;
  const MessagesCompanion({
    this.id = const Value.absent(),
    this.conversationId = const Value.absent(),
    this.role = const Value.absent(),
    this.content = const Value.absent(),
    this.displayName = const Value.absent(),
    this.thought = const Value.absent(),
    this.imageUrl = const Value.absent(),
    this.videoUrl = const Value.absent(),
    this.audioUrl = const Value.absent(),
    this.documentUrl = const Value.absent(),
    this.stickerUrl = const Value.absent(),
    this.userId = const Value.absent(),
    this.totalTokens = const Value.absent(),
    this.createdAt = const Value.absent(),
    this.metadata = const Value.absent(),
    this.replyToId = const Value.absent(),
    this.repliedMessage = const Value.absent(),
    this.syncStatus = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  MessagesCompanion.insert({
    required String id,
    required String conversationId,
    required String role,
    required String content,
    this.displayName = const Value.absent(),
    this.thought = const Value.absent(),
    this.imageUrl = const Value.absent(),
    this.videoUrl = const Value.absent(),
    this.audioUrl = const Value.absent(),
    this.documentUrl = const Value.absent(),
    this.stickerUrl = const Value.absent(),
    this.userId = const Value.absent(),
    this.totalTokens = const Value.absent(),
    required DateTime createdAt,
    this.metadata = const Value.absent(),
    this.replyToId = const Value.absent(),
    this.repliedMessage = const Value.absent(),
    required SyncStatus syncStatus,
    this.rowid = const Value.absent(),
  }) : id = Value(id),
       conversationId = Value(conversationId),
       role = Value(role),
       content = Value(content),
       createdAt = Value(createdAt),
       syncStatus = Value(syncStatus);
  static Insertable<Message> custom({
    Expression<String>? id,
    Expression<String>? conversationId,
    Expression<String>? role,
    Expression<String>? content,
    Expression<String>? displayName,
    Expression<String>? thought,
    Expression<String>? imageUrl,
    Expression<String>? videoUrl,
    Expression<String>? audioUrl,
    Expression<String>? documentUrl,
    Expression<String>? stickerUrl,
    Expression<String>? userId,
    Expression<int>? totalTokens,
    Expression<DateTime>? createdAt,
    Expression<String>? metadata,
    Expression<String>? replyToId,
    Expression<String>? repliedMessage,
    Expression<int>? syncStatus,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (conversationId != null) 'conversation_id': conversationId,
      if (role != null) 'role': role,
      if (content != null) 'content': content,
      if (displayName != null) 'display_name': displayName,
      if (thought != null) 'thought': thought,
      if (imageUrl != null) 'image_url': imageUrl,
      if (videoUrl != null) 'video_url': videoUrl,
      if (audioUrl != null) 'audio_url': audioUrl,
      if (documentUrl != null) 'document_url': documentUrl,
      if (stickerUrl != null) 'sticker_url': stickerUrl,
      if (userId != null) 'user_id': userId,
      if (totalTokens != null) 'total_tokens': totalTokens,
      if (createdAt != null) 'created_at': createdAt,
      if (metadata != null) 'metadata': metadata,
      if (replyToId != null) 'reply_to_id': replyToId,
      if (repliedMessage != null) 'replied_message': repliedMessage,
      if (syncStatus != null) 'sync_status': syncStatus,
      if (rowid != null) 'rowid': rowid,
    });
  }

  MessagesCompanion copyWith({
    Value<String>? id,
    Value<String>? conversationId,
    Value<String>? role,
    Value<String>? content,
    Value<String?>? displayName,
    Value<String?>? thought,
    Value<String?>? imageUrl,
    Value<String?>? videoUrl,
    Value<String?>? audioUrl,
    Value<String?>? documentUrl,
    Value<String?>? stickerUrl,
    Value<String?>? userId,
    Value<int?>? totalTokens,
    Value<DateTime>? createdAt,
    Value<String?>? metadata,
    Value<String?>? replyToId,
    Value<String?>? repliedMessage,
    Value<SyncStatus>? syncStatus,
    Value<int>? rowid,
  }) {
    return MessagesCompanion(
      id: id ?? this.id,
      conversationId: conversationId ?? this.conversationId,
      role: role ?? this.role,
      content: content ?? this.content,
      displayName: displayName ?? this.displayName,
      thought: thought ?? this.thought,
      imageUrl: imageUrl ?? this.imageUrl,
      videoUrl: videoUrl ?? this.videoUrl,
      audioUrl: audioUrl ?? this.audioUrl,
      documentUrl: documentUrl ?? this.documentUrl,
      stickerUrl: stickerUrl ?? this.stickerUrl,
      userId: userId ?? this.userId,
      totalTokens: totalTokens ?? this.totalTokens,
      createdAt: createdAt ?? this.createdAt,
      metadata: metadata ?? this.metadata,
      replyToId: replyToId ?? this.replyToId,
      repliedMessage: repliedMessage ?? this.repliedMessage,
      syncStatus: syncStatus ?? this.syncStatus,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<String>(id.value);
    }
    if (conversationId.present) {
      map['conversation_id'] = Variable<String>(conversationId.value);
    }
    if (role.present) {
      map['role'] = Variable<String>(role.value);
    }
    if (content.present) {
      map['content'] = Variable<String>(content.value);
    }
    if (displayName.present) {
      map['display_name'] = Variable<String>(displayName.value);
    }
    if (thought.present) {
      map['thought'] = Variable<String>(thought.value);
    }
    if (imageUrl.present) {
      map['image_url'] = Variable<String>(imageUrl.value);
    }
    if (videoUrl.present) {
      map['video_url'] = Variable<String>(videoUrl.value);
    }
    if (audioUrl.present) {
      map['audio_url'] = Variable<String>(audioUrl.value);
    }
    if (documentUrl.present) {
      map['document_url'] = Variable<String>(documentUrl.value);
    }
    if (stickerUrl.present) {
      map['sticker_url'] = Variable<String>(stickerUrl.value);
    }
    if (userId.present) {
      map['user_id'] = Variable<String>(userId.value);
    }
    if (totalTokens.present) {
      map['total_tokens'] = Variable<int>(totalTokens.value);
    }
    if (createdAt.present) {
      map['created_at'] = Variable<DateTime>(createdAt.value);
    }
    if (metadata.present) {
      map['metadata'] = Variable<String>(metadata.value);
    }
    if (replyToId.present) {
      map['reply_to_id'] = Variable<String>(replyToId.value);
    }
    if (repliedMessage.present) {
      map['replied_message'] = Variable<String>(repliedMessage.value);
    }
    if (syncStatus.present) {
      map['sync_status'] = Variable<int>(
        $MessagesTable.$convertersyncStatus.toSql(syncStatus.value),
      );
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('MessagesCompanion(')
          ..write('id: $id, ')
          ..write('conversationId: $conversationId, ')
          ..write('role: $role, ')
          ..write('content: $content, ')
          ..write('displayName: $displayName, ')
          ..write('thought: $thought, ')
          ..write('imageUrl: $imageUrl, ')
          ..write('videoUrl: $videoUrl, ')
          ..write('audioUrl: $audioUrl, ')
          ..write('documentUrl: $documentUrl, ')
          ..write('stickerUrl: $stickerUrl, ')
          ..write('userId: $userId, ')
          ..write('totalTokens: $totalTokens, ')
          ..write('createdAt: $createdAt, ')
          ..write('metadata: $metadata, ')
          ..write('replyToId: $replyToId, ')
          ..write('repliedMessage: $repliedMessage, ')
          ..write('syncStatus: $syncStatus, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

class $RemindersTable extends Reminders
    with TableInfo<$RemindersTable, Reminder> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $RemindersTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<String> id = GeneratedColumn<String>(
    'id',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _contentMeta = const VerificationMeta(
    'content',
  );
  @override
  late final GeneratedColumn<String> content = GeneratedColumn<String>(
    'content',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _taskTypeMeta = const VerificationMeta(
    'taskType',
  );
  @override
  late final GeneratedColumn<String> taskType = GeneratedColumn<String>(
    'task_type',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _frequencyMeta = const VerificationMeta(
    'frequency',
  );
  @override
  late final GeneratedColumn<String> frequency = GeneratedColumn<String>(
    'frequency',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _statusMeta = const VerificationMeta('status');
  @override
  late final GeneratedColumn<String> status = GeneratedColumn<String>(
    'status',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _dueAtMeta = const VerificationMeta('dueAt');
  @override
  late final GeneratedColumn<DateTime> dueAt = GeneratedColumn<DateTime>(
    'due_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _createdAtMeta = const VerificationMeta(
    'createdAt',
  );
  @override
  late final GeneratedColumn<DateTime> createdAt = GeneratedColumn<DateTime>(
    'created_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _userDisplayNameMeta = const VerificationMeta(
    'userDisplayName',
  );
  @override
  late final GeneratedColumn<String> userDisplayName = GeneratedColumn<String>(
    'user_display_name',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _conversationTitleMeta = const VerificationMeta(
    'conversationTitle',
  );
  @override
  late final GeneratedColumn<String> conversationTitle =
      GeneratedColumn<String>(
        'conversation_title',
        aliasedName,
        true,
        type: DriftSqlType.string,
        requiredDuringInsert: false,
      );
  @override
  List<GeneratedColumn> get $columns => [
    id,
    content,
    taskType,
    frequency,
    status,
    dueAt,
    createdAt,
    userDisplayName,
    conversationTitle,
  ];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'reminders';
  @override
  VerificationContext validateIntegrity(
    Insertable<Reminder> instance, {
    bool isInserting = false,
  }) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('content')) {
      context.handle(
        _contentMeta,
        content.isAcceptableOrUnknown(data['content']!, _contentMeta),
      );
    } else if (isInserting) {
      context.missing(_contentMeta);
    }
    if (data.containsKey('task_type')) {
      context.handle(
        _taskTypeMeta,
        taskType.isAcceptableOrUnknown(data['task_type']!, _taskTypeMeta),
      );
    }
    if (data.containsKey('frequency')) {
      context.handle(
        _frequencyMeta,
        frequency.isAcceptableOrUnknown(data['frequency']!, _frequencyMeta),
      );
    }
    if (data.containsKey('status')) {
      context.handle(
        _statusMeta,
        status.isAcceptableOrUnknown(data['status']!, _statusMeta),
      );
    } else if (isInserting) {
      context.missing(_statusMeta);
    }
    if (data.containsKey('due_at')) {
      context.handle(
        _dueAtMeta,
        dueAt.isAcceptableOrUnknown(data['due_at']!, _dueAtMeta),
      );
    } else if (isInserting) {
      context.missing(_dueAtMeta);
    }
    if (data.containsKey('created_at')) {
      context.handle(
        _createdAtMeta,
        createdAt.isAcceptableOrUnknown(data['created_at']!, _createdAtMeta),
      );
    } else if (isInserting) {
      context.missing(_createdAtMeta);
    }
    if (data.containsKey('user_display_name')) {
      context.handle(
        _userDisplayNameMeta,
        userDisplayName.isAcceptableOrUnknown(
          data['user_display_name']!,
          _userDisplayNameMeta,
        ),
      );
    }
    if (data.containsKey('conversation_title')) {
      context.handle(
        _conversationTitleMeta,
        conversationTitle.isAcceptableOrUnknown(
          data['conversation_title']!,
          _conversationTitleMeta,
        ),
      );
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  Reminder map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return Reminder(
      id:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}id'],
          )!,
      content:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}content'],
          )!,
      taskType: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}task_type'],
      ),
      frequency: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}frequency'],
      ),
      status:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}status'],
          )!,
      dueAt:
          attachedDatabase.typeMapping.read(
            DriftSqlType.dateTime,
            data['${effectivePrefix}due_at'],
          )!,
      createdAt:
          attachedDatabase.typeMapping.read(
            DriftSqlType.dateTime,
            data['${effectivePrefix}created_at'],
          )!,
      userDisplayName: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}user_display_name'],
      ),
      conversationTitle: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}conversation_title'],
      ),
    );
  }

  @override
  $RemindersTable createAlias(String alias) {
    return $RemindersTable(attachedDatabase, alias);
  }
}

class Reminder extends DataClass implements Insertable<Reminder> {
  final String id;
  final String content;
  final String? taskType;
  final String? frequency;
  final String status;
  final DateTime dueAt;
  final DateTime createdAt;
  final String? userDisplayName;
  final String? conversationTitle;
  const Reminder({
    required this.id,
    required this.content,
    this.taskType,
    this.frequency,
    required this.status,
    required this.dueAt,
    required this.createdAt,
    this.userDisplayName,
    this.conversationTitle,
  });
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    map['content'] = Variable<String>(content);
    if (!nullToAbsent || taskType != null) {
      map['task_type'] = Variable<String>(taskType);
    }
    if (!nullToAbsent || frequency != null) {
      map['frequency'] = Variable<String>(frequency);
    }
    map['status'] = Variable<String>(status);
    map['due_at'] = Variable<DateTime>(dueAt);
    map['created_at'] = Variable<DateTime>(createdAt);
    if (!nullToAbsent || userDisplayName != null) {
      map['user_display_name'] = Variable<String>(userDisplayName);
    }
    if (!nullToAbsent || conversationTitle != null) {
      map['conversation_title'] = Variable<String>(conversationTitle);
    }
    return map;
  }

  RemindersCompanion toCompanion(bool nullToAbsent) {
    return RemindersCompanion(
      id: Value(id),
      content: Value(content),
      taskType:
          taskType == null && nullToAbsent
              ? const Value.absent()
              : Value(taskType),
      frequency:
          frequency == null && nullToAbsent
              ? const Value.absent()
              : Value(frequency),
      status: Value(status),
      dueAt: Value(dueAt),
      createdAt: Value(createdAt),
      userDisplayName:
          userDisplayName == null && nullToAbsent
              ? const Value.absent()
              : Value(userDisplayName),
      conversationTitle:
          conversationTitle == null && nullToAbsent
              ? const Value.absent()
              : Value(conversationTitle),
    );
  }

  factory Reminder.fromJson(
    Map<String, dynamic> json, {
    ValueSerializer? serializer,
  }) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return Reminder(
      id: serializer.fromJson<String>(json['id']),
      content: serializer.fromJson<String>(json['content']),
      taskType: serializer.fromJson<String?>(json['taskType']),
      frequency: serializer.fromJson<String?>(json['frequency']),
      status: serializer.fromJson<String>(json['status']),
      dueAt: serializer.fromJson<DateTime>(json['dueAt']),
      createdAt: serializer.fromJson<DateTime>(json['createdAt']),
      userDisplayName: serializer.fromJson<String?>(json['userDisplayName']),
      conversationTitle: serializer.fromJson<String?>(
        json['conversationTitle'],
      ),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<String>(id),
      'content': serializer.toJson<String>(content),
      'taskType': serializer.toJson<String?>(taskType),
      'frequency': serializer.toJson<String?>(frequency),
      'status': serializer.toJson<String>(status),
      'dueAt': serializer.toJson<DateTime>(dueAt),
      'createdAt': serializer.toJson<DateTime>(createdAt),
      'userDisplayName': serializer.toJson<String?>(userDisplayName),
      'conversationTitle': serializer.toJson<String?>(conversationTitle),
    };
  }

  Reminder copyWith({
    String? id,
    String? content,
    Value<String?> taskType = const Value.absent(),
    Value<String?> frequency = const Value.absent(),
    String? status,
    DateTime? dueAt,
    DateTime? createdAt,
    Value<String?> userDisplayName = const Value.absent(),
    Value<String?> conversationTitle = const Value.absent(),
  }) => Reminder(
    id: id ?? this.id,
    content: content ?? this.content,
    taskType: taskType.present ? taskType.value : this.taskType,
    frequency: frequency.present ? frequency.value : this.frequency,
    status: status ?? this.status,
    dueAt: dueAt ?? this.dueAt,
    createdAt: createdAt ?? this.createdAt,
    userDisplayName:
        userDisplayName.present ? userDisplayName.value : this.userDisplayName,
    conversationTitle:
        conversationTitle.present
            ? conversationTitle.value
            : this.conversationTitle,
  );
  Reminder copyWithCompanion(RemindersCompanion data) {
    return Reminder(
      id: data.id.present ? data.id.value : this.id,
      content: data.content.present ? data.content.value : this.content,
      taskType: data.taskType.present ? data.taskType.value : this.taskType,
      frequency: data.frequency.present ? data.frequency.value : this.frequency,
      status: data.status.present ? data.status.value : this.status,
      dueAt: data.dueAt.present ? data.dueAt.value : this.dueAt,
      createdAt: data.createdAt.present ? data.createdAt.value : this.createdAt,
      userDisplayName:
          data.userDisplayName.present
              ? data.userDisplayName.value
              : this.userDisplayName,
      conversationTitle:
          data.conversationTitle.present
              ? data.conversationTitle.value
              : this.conversationTitle,
    );
  }

  @override
  String toString() {
    return (StringBuffer('Reminder(')
          ..write('id: $id, ')
          ..write('content: $content, ')
          ..write('taskType: $taskType, ')
          ..write('frequency: $frequency, ')
          ..write('status: $status, ')
          ..write('dueAt: $dueAt, ')
          ..write('createdAt: $createdAt, ')
          ..write('userDisplayName: $userDisplayName, ')
          ..write('conversationTitle: $conversationTitle')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(
    id,
    content,
    taskType,
    frequency,
    status,
    dueAt,
    createdAt,
    userDisplayName,
    conversationTitle,
  );
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Reminder &&
          other.id == this.id &&
          other.content == this.content &&
          other.taskType == this.taskType &&
          other.frequency == this.frequency &&
          other.status == this.status &&
          other.dueAt == this.dueAt &&
          other.createdAt == this.createdAt &&
          other.userDisplayName == this.userDisplayName &&
          other.conversationTitle == this.conversationTitle);
}

class RemindersCompanion extends UpdateCompanion<Reminder> {
  final Value<String> id;
  final Value<String> content;
  final Value<String?> taskType;
  final Value<String?> frequency;
  final Value<String> status;
  final Value<DateTime> dueAt;
  final Value<DateTime> createdAt;
  final Value<String?> userDisplayName;
  final Value<String?> conversationTitle;
  final Value<int> rowid;
  const RemindersCompanion({
    this.id = const Value.absent(),
    this.content = const Value.absent(),
    this.taskType = const Value.absent(),
    this.frequency = const Value.absent(),
    this.status = const Value.absent(),
    this.dueAt = const Value.absent(),
    this.createdAt = const Value.absent(),
    this.userDisplayName = const Value.absent(),
    this.conversationTitle = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  RemindersCompanion.insert({
    required String id,
    required String content,
    this.taskType = const Value.absent(),
    this.frequency = const Value.absent(),
    required String status,
    required DateTime dueAt,
    required DateTime createdAt,
    this.userDisplayName = const Value.absent(),
    this.conversationTitle = const Value.absent(),
    this.rowid = const Value.absent(),
  }) : id = Value(id),
       content = Value(content),
       status = Value(status),
       dueAt = Value(dueAt),
       createdAt = Value(createdAt);
  static Insertable<Reminder> custom({
    Expression<String>? id,
    Expression<String>? content,
    Expression<String>? taskType,
    Expression<String>? frequency,
    Expression<String>? status,
    Expression<DateTime>? dueAt,
    Expression<DateTime>? createdAt,
    Expression<String>? userDisplayName,
    Expression<String>? conversationTitle,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (content != null) 'content': content,
      if (taskType != null) 'task_type': taskType,
      if (frequency != null) 'frequency': frequency,
      if (status != null) 'status': status,
      if (dueAt != null) 'due_at': dueAt,
      if (createdAt != null) 'created_at': createdAt,
      if (userDisplayName != null) 'user_display_name': userDisplayName,
      if (conversationTitle != null) 'conversation_title': conversationTitle,
      if (rowid != null) 'rowid': rowid,
    });
  }

  RemindersCompanion copyWith({
    Value<String>? id,
    Value<String>? content,
    Value<String?>? taskType,
    Value<String?>? frequency,
    Value<String>? status,
    Value<DateTime>? dueAt,
    Value<DateTime>? createdAt,
    Value<String?>? userDisplayName,
    Value<String?>? conversationTitle,
    Value<int>? rowid,
  }) {
    return RemindersCompanion(
      id: id ?? this.id,
      content: content ?? this.content,
      taskType: taskType ?? this.taskType,
      frequency: frequency ?? this.frequency,
      status: status ?? this.status,
      dueAt: dueAt ?? this.dueAt,
      createdAt: createdAt ?? this.createdAt,
      userDisplayName: userDisplayName ?? this.userDisplayName,
      conversationTitle: conversationTitle ?? this.conversationTitle,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<String>(id.value);
    }
    if (content.present) {
      map['content'] = Variable<String>(content.value);
    }
    if (taskType.present) {
      map['task_type'] = Variable<String>(taskType.value);
    }
    if (frequency.present) {
      map['frequency'] = Variable<String>(frequency.value);
    }
    if (status.present) {
      map['status'] = Variable<String>(status.value);
    }
    if (dueAt.present) {
      map['due_at'] = Variable<DateTime>(dueAt.value);
    }
    if (createdAt.present) {
      map['created_at'] = Variable<DateTime>(createdAt.value);
    }
    if (userDisplayName.present) {
      map['user_display_name'] = Variable<String>(userDisplayName.value);
    }
    if (conversationTitle.present) {
      map['conversation_title'] = Variable<String>(conversationTitle.value);
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('RemindersCompanion(')
          ..write('id: $id, ')
          ..write('content: $content, ')
          ..write('taskType: $taskType, ')
          ..write('frequency: $frequency, ')
          ..write('status: $status, ')
          ..write('dueAt: $dueAt, ')
          ..write('createdAt: $createdAt, ')
          ..write('userDisplayName: $userDisplayName, ')
          ..write('conversationTitle: $conversationTitle, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

class $TransactionsTable extends Transactions
    with TableInfo<$TransactionsTable, Transaction> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $TransactionsTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<String> id = GeneratedColumn<String>(
    'id',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _merchantNameMeta = const VerificationMeta(
    'merchantName',
  );
  @override
  late final GeneratedColumn<String> merchantName = GeneratedColumn<String>(
    'merchant_name',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _categoryMeta = const VerificationMeta(
    'category',
  );
  @override
  late final GeneratedColumn<String> category = GeneratedColumn<String>(
    'category',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _descriptionMeta = const VerificationMeta(
    'description',
  );
  @override
  late final GeneratedColumn<String> description = GeneratedColumn<String>(
    'description',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _totalAmountMeta = const VerificationMeta(
    'totalAmount',
  );
  @override
  late final GeneratedColumn<String> totalAmount = GeneratedColumn<String>(
    'total_amount',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _createdAtMeta = const VerificationMeta(
    'createdAt',
  );
  @override
  late final GeneratedColumn<DateTime> createdAt = GeneratedColumn<DateTime>(
    'created_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _userDisplayNameMeta = const VerificationMeta(
    'userDisplayName',
  );
  @override
  late final GeneratedColumn<String> userDisplayName = GeneratedColumn<String>(
    'user_display_name',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _conversationTitleMeta = const VerificationMeta(
    'conversationTitle',
  );
  @override
  late final GeneratedColumn<String> conversationTitle =
      GeneratedColumn<String>(
        'conversation_title',
        aliasedName,
        true,
        type: DriftSqlType.string,
        requiredDuringInsert: false,
      );
  static const VerificationMeta _itemsJsonMeta = const VerificationMeta(
    'itemsJson',
  );
  @override
  late final GeneratedColumn<String> itemsJson = GeneratedColumn<String>(
    'items_json',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  @override
  List<GeneratedColumn> get $columns => [
    id,
    merchantName,
    category,
    description,
    totalAmount,
    createdAt,
    userDisplayName,
    conversationTitle,
    itemsJson,
  ];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'transactions';
  @override
  VerificationContext validateIntegrity(
    Insertable<Transaction> instance, {
    bool isInserting = false,
  }) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('merchant_name')) {
      context.handle(
        _merchantNameMeta,
        merchantName.isAcceptableOrUnknown(
          data['merchant_name']!,
          _merchantNameMeta,
        ),
      );
    }
    if (data.containsKey('category')) {
      context.handle(
        _categoryMeta,
        category.isAcceptableOrUnknown(data['category']!, _categoryMeta),
      );
    }
    if (data.containsKey('description')) {
      context.handle(
        _descriptionMeta,
        description.isAcceptableOrUnknown(
          data['description']!,
          _descriptionMeta,
        ),
      );
    }
    if (data.containsKey('total_amount')) {
      context.handle(
        _totalAmountMeta,
        totalAmount.isAcceptableOrUnknown(
          data['total_amount']!,
          _totalAmountMeta,
        ),
      );
    } else if (isInserting) {
      context.missing(_totalAmountMeta);
    }
    if (data.containsKey('created_at')) {
      context.handle(
        _createdAtMeta,
        createdAt.isAcceptableOrUnknown(data['created_at']!, _createdAtMeta),
      );
    } else if (isInserting) {
      context.missing(_createdAtMeta);
    }
    if (data.containsKey('user_display_name')) {
      context.handle(
        _userDisplayNameMeta,
        userDisplayName.isAcceptableOrUnknown(
          data['user_display_name']!,
          _userDisplayNameMeta,
        ),
      );
    }
    if (data.containsKey('conversation_title')) {
      context.handle(
        _conversationTitleMeta,
        conversationTitle.isAcceptableOrUnknown(
          data['conversation_title']!,
          _conversationTitleMeta,
        ),
      );
    }
    if (data.containsKey('items_json')) {
      context.handle(
        _itemsJsonMeta,
        itemsJson.isAcceptableOrUnknown(data['items_json']!, _itemsJsonMeta),
      );
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  Transaction map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return Transaction(
      id:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}id'],
          )!,
      merchantName: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}merchant_name'],
      ),
      category: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}category'],
      ),
      description: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}description'],
      ),
      totalAmount:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}total_amount'],
          )!,
      createdAt:
          attachedDatabase.typeMapping.read(
            DriftSqlType.dateTime,
            data['${effectivePrefix}created_at'],
          )!,
      userDisplayName: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}user_display_name'],
      ),
      conversationTitle: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}conversation_title'],
      ),
      itemsJson: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}items_json'],
      ),
    );
  }

  @override
  $TransactionsTable createAlias(String alias) {
    return $TransactionsTable(attachedDatabase, alias);
  }
}

class Transaction extends DataClass implements Insertable<Transaction> {
  final String id;
  final String? merchantName;
  final String? category;
  final String? description;
  final String totalAmount;
  final DateTime createdAt;
  final String? userDisplayName;
  final String? conversationTitle;
  final String? itemsJson;
  const Transaction({
    required this.id,
    this.merchantName,
    this.category,
    this.description,
    required this.totalAmount,
    required this.createdAt,
    this.userDisplayName,
    this.conversationTitle,
    this.itemsJson,
  });
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    if (!nullToAbsent || merchantName != null) {
      map['merchant_name'] = Variable<String>(merchantName);
    }
    if (!nullToAbsent || category != null) {
      map['category'] = Variable<String>(category);
    }
    if (!nullToAbsent || description != null) {
      map['description'] = Variable<String>(description);
    }
    map['total_amount'] = Variable<String>(totalAmount);
    map['created_at'] = Variable<DateTime>(createdAt);
    if (!nullToAbsent || userDisplayName != null) {
      map['user_display_name'] = Variable<String>(userDisplayName);
    }
    if (!nullToAbsent || conversationTitle != null) {
      map['conversation_title'] = Variable<String>(conversationTitle);
    }
    if (!nullToAbsent || itemsJson != null) {
      map['items_json'] = Variable<String>(itemsJson);
    }
    return map;
  }

  TransactionsCompanion toCompanion(bool nullToAbsent) {
    return TransactionsCompanion(
      id: Value(id),
      merchantName:
          merchantName == null && nullToAbsent
              ? const Value.absent()
              : Value(merchantName),
      category:
          category == null && nullToAbsent
              ? const Value.absent()
              : Value(category),
      description:
          description == null && nullToAbsent
              ? const Value.absent()
              : Value(description),
      totalAmount: Value(totalAmount),
      createdAt: Value(createdAt),
      userDisplayName:
          userDisplayName == null && nullToAbsent
              ? const Value.absent()
              : Value(userDisplayName),
      conversationTitle:
          conversationTitle == null && nullToAbsent
              ? const Value.absent()
              : Value(conversationTitle),
      itemsJson:
          itemsJson == null && nullToAbsent
              ? const Value.absent()
              : Value(itemsJson),
    );
  }

  factory Transaction.fromJson(
    Map<String, dynamic> json, {
    ValueSerializer? serializer,
  }) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return Transaction(
      id: serializer.fromJson<String>(json['id']),
      merchantName: serializer.fromJson<String?>(json['merchantName']),
      category: serializer.fromJson<String?>(json['category']),
      description: serializer.fromJson<String?>(json['description']),
      totalAmount: serializer.fromJson<String>(json['totalAmount']),
      createdAt: serializer.fromJson<DateTime>(json['createdAt']),
      userDisplayName: serializer.fromJson<String?>(json['userDisplayName']),
      conversationTitle: serializer.fromJson<String?>(
        json['conversationTitle'],
      ),
      itemsJson: serializer.fromJson<String?>(json['itemsJson']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<String>(id),
      'merchantName': serializer.toJson<String?>(merchantName),
      'category': serializer.toJson<String?>(category),
      'description': serializer.toJson<String?>(description),
      'totalAmount': serializer.toJson<String>(totalAmount),
      'createdAt': serializer.toJson<DateTime>(createdAt),
      'userDisplayName': serializer.toJson<String?>(userDisplayName),
      'conversationTitle': serializer.toJson<String?>(conversationTitle),
      'itemsJson': serializer.toJson<String?>(itemsJson),
    };
  }

  Transaction copyWith({
    String? id,
    Value<String?> merchantName = const Value.absent(),
    Value<String?> category = const Value.absent(),
    Value<String?> description = const Value.absent(),
    String? totalAmount,
    DateTime? createdAt,
    Value<String?> userDisplayName = const Value.absent(),
    Value<String?> conversationTitle = const Value.absent(),
    Value<String?> itemsJson = const Value.absent(),
  }) => Transaction(
    id: id ?? this.id,
    merchantName: merchantName.present ? merchantName.value : this.merchantName,
    category: category.present ? category.value : this.category,
    description: description.present ? description.value : this.description,
    totalAmount: totalAmount ?? this.totalAmount,
    createdAt: createdAt ?? this.createdAt,
    userDisplayName:
        userDisplayName.present ? userDisplayName.value : this.userDisplayName,
    conversationTitle:
        conversationTitle.present
            ? conversationTitle.value
            : this.conversationTitle,
    itemsJson: itemsJson.present ? itemsJson.value : this.itemsJson,
  );
  Transaction copyWithCompanion(TransactionsCompanion data) {
    return Transaction(
      id: data.id.present ? data.id.value : this.id,
      merchantName:
          data.merchantName.present
              ? data.merchantName.value
              : this.merchantName,
      category: data.category.present ? data.category.value : this.category,
      description:
          data.description.present ? data.description.value : this.description,
      totalAmount:
          data.totalAmount.present ? data.totalAmount.value : this.totalAmount,
      createdAt: data.createdAt.present ? data.createdAt.value : this.createdAt,
      userDisplayName:
          data.userDisplayName.present
              ? data.userDisplayName.value
              : this.userDisplayName,
      conversationTitle:
          data.conversationTitle.present
              ? data.conversationTitle.value
              : this.conversationTitle,
      itemsJson: data.itemsJson.present ? data.itemsJson.value : this.itemsJson,
    );
  }

  @override
  String toString() {
    return (StringBuffer('Transaction(')
          ..write('id: $id, ')
          ..write('merchantName: $merchantName, ')
          ..write('category: $category, ')
          ..write('description: $description, ')
          ..write('totalAmount: $totalAmount, ')
          ..write('createdAt: $createdAt, ')
          ..write('userDisplayName: $userDisplayName, ')
          ..write('conversationTitle: $conversationTitle, ')
          ..write('itemsJson: $itemsJson')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(
    id,
    merchantName,
    category,
    description,
    totalAmount,
    createdAt,
    userDisplayName,
    conversationTitle,
    itemsJson,
  );
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Transaction &&
          other.id == this.id &&
          other.merchantName == this.merchantName &&
          other.category == this.category &&
          other.description == this.description &&
          other.totalAmount == this.totalAmount &&
          other.createdAt == this.createdAt &&
          other.userDisplayName == this.userDisplayName &&
          other.conversationTitle == this.conversationTitle &&
          other.itemsJson == this.itemsJson);
}

class TransactionsCompanion extends UpdateCompanion<Transaction> {
  final Value<String> id;
  final Value<String?> merchantName;
  final Value<String?> category;
  final Value<String?> description;
  final Value<String> totalAmount;
  final Value<DateTime> createdAt;
  final Value<String?> userDisplayName;
  final Value<String?> conversationTitle;
  final Value<String?> itemsJson;
  final Value<int> rowid;
  const TransactionsCompanion({
    this.id = const Value.absent(),
    this.merchantName = const Value.absent(),
    this.category = const Value.absent(),
    this.description = const Value.absent(),
    this.totalAmount = const Value.absent(),
    this.createdAt = const Value.absent(),
    this.userDisplayName = const Value.absent(),
    this.conversationTitle = const Value.absent(),
    this.itemsJson = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  TransactionsCompanion.insert({
    required String id,
    this.merchantName = const Value.absent(),
    this.category = const Value.absent(),
    this.description = const Value.absent(),
    required String totalAmount,
    required DateTime createdAt,
    this.userDisplayName = const Value.absent(),
    this.conversationTitle = const Value.absent(),
    this.itemsJson = const Value.absent(),
    this.rowid = const Value.absent(),
  }) : id = Value(id),
       totalAmount = Value(totalAmount),
       createdAt = Value(createdAt);
  static Insertable<Transaction> custom({
    Expression<String>? id,
    Expression<String>? merchantName,
    Expression<String>? category,
    Expression<String>? description,
    Expression<String>? totalAmount,
    Expression<DateTime>? createdAt,
    Expression<String>? userDisplayName,
    Expression<String>? conversationTitle,
    Expression<String>? itemsJson,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (merchantName != null) 'merchant_name': merchantName,
      if (category != null) 'category': category,
      if (description != null) 'description': description,
      if (totalAmount != null) 'total_amount': totalAmount,
      if (createdAt != null) 'created_at': createdAt,
      if (userDisplayName != null) 'user_display_name': userDisplayName,
      if (conversationTitle != null) 'conversation_title': conversationTitle,
      if (itemsJson != null) 'items_json': itemsJson,
      if (rowid != null) 'rowid': rowid,
    });
  }

  TransactionsCompanion copyWith({
    Value<String>? id,
    Value<String?>? merchantName,
    Value<String?>? category,
    Value<String?>? description,
    Value<String>? totalAmount,
    Value<DateTime>? createdAt,
    Value<String?>? userDisplayName,
    Value<String?>? conversationTitle,
    Value<String?>? itemsJson,
    Value<int>? rowid,
  }) {
    return TransactionsCompanion(
      id: id ?? this.id,
      merchantName: merchantName ?? this.merchantName,
      category: category ?? this.category,
      description: description ?? this.description,
      totalAmount: totalAmount ?? this.totalAmount,
      createdAt: createdAt ?? this.createdAt,
      userDisplayName: userDisplayName ?? this.userDisplayName,
      conversationTitle: conversationTitle ?? this.conversationTitle,
      itemsJson: itemsJson ?? this.itemsJson,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<String>(id.value);
    }
    if (merchantName.present) {
      map['merchant_name'] = Variable<String>(merchantName.value);
    }
    if (category.present) {
      map['category'] = Variable<String>(category.value);
    }
    if (description.present) {
      map['description'] = Variable<String>(description.value);
    }
    if (totalAmount.present) {
      map['total_amount'] = Variable<String>(totalAmount.value);
    }
    if (createdAt.present) {
      map['created_at'] = Variable<DateTime>(createdAt.value);
    }
    if (userDisplayName.present) {
      map['user_display_name'] = Variable<String>(userDisplayName.value);
    }
    if (conversationTitle.present) {
      map['conversation_title'] = Variable<String>(conversationTitle.value);
    }
    if (itemsJson.present) {
      map['items_json'] = Variable<String>(itemsJson.value);
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('TransactionsCompanion(')
          ..write('id: $id, ')
          ..write('merchantName: $merchantName, ')
          ..write('category: $category, ')
          ..write('description: $description, ')
          ..write('totalAmount: $totalAmount, ')
          ..write('createdAt: $createdAt, ')
          ..write('userDisplayName: $userDisplayName, ')
          ..write('conversationTitle: $conversationTitle, ')
          ..write('itemsJson: $itemsJson, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

class $HealthMetricsTable extends HealthMetrics
    with TableInfo<$HealthMetricsTable, HealthMetric> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $HealthMetricsTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<String> id = GeneratedColumn<String>(
    'id',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _userIdMeta = const VerificationMeta('userId');
  @override
  late final GeneratedColumn<String> userId = GeneratedColumn<String>(
    'user_id',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _logDateMeta = const VerificationMeta(
    'logDate',
  );
  @override
  late final GeneratedColumn<DateTime> logDate = GeneratedColumn<DateTime>(
    'log_date',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _stepsMeta = const VerificationMeta('steps');
  @override
  late final GeneratedColumn<int> steps = GeneratedColumn<int>(
    'steps',
    aliasedName,
    false,
    type: DriftSqlType.int,
    requiredDuringInsert: false,
    defaultValue: const Constant(0),
  );
  static const VerificationMeta _avgHeartRateMeta = const VerificationMeta(
    'avgHeartRate',
  );
  @override
  late final GeneratedColumn<int> avgHeartRate = GeneratedColumn<int>(
    'avg_heart_rate',
    aliasedName,
    true,
    type: DriftSqlType.int,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _sleepHoursMeta = const VerificationMeta(
    'sleepHours',
  );
  @override
  late final GeneratedColumn<double> sleepHours = GeneratedColumn<double>(
    'sleep_hours',
    aliasedName,
    true,
    type: DriftSqlType.double,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _updatedAtMeta = const VerificationMeta(
    'updatedAt',
  );
  @override
  late final GeneratedColumn<DateTime> updatedAt = GeneratedColumn<DateTime>(
    'updated_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  @override
  List<GeneratedColumn> get $columns => [
    id,
    userId,
    logDate,
    steps,
    avgHeartRate,
    sleepHours,
    updatedAt,
  ];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'health_metrics';
  @override
  VerificationContext validateIntegrity(
    Insertable<HealthMetric> instance, {
    bool isInserting = false,
  }) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('user_id')) {
      context.handle(
        _userIdMeta,
        userId.isAcceptableOrUnknown(data['user_id']!, _userIdMeta),
      );
    } else if (isInserting) {
      context.missing(_userIdMeta);
    }
    if (data.containsKey('log_date')) {
      context.handle(
        _logDateMeta,
        logDate.isAcceptableOrUnknown(data['log_date']!, _logDateMeta),
      );
    } else if (isInserting) {
      context.missing(_logDateMeta);
    }
    if (data.containsKey('steps')) {
      context.handle(
        _stepsMeta,
        steps.isAcceptableOrUnknown(data['steps']!, _stepsMeta),
      );
    }
    if (data.containsKey('avg_heart_rate')) {
      context.handle(
        _avgHeartRateMeta,
        avgHeartRate.isAcceptableOrUnknown(
          data['avg_heart_rate']!,
          _avgHeartRateMeta,
        ),
      );
    }
    if (data.containsKey('sleep_hours')) {
      context.handle(
        _sleepHoursMeta,
        sleepHours.isAcceptableOrUnknown(data['sleep_hours']!, _sleepHoursMeta),
      );
    }
    if (data.containsKey('updated_at')) {
      context.handle(
        _updatedAtMeta,
        updatedAt.isAcceptableOrUnknown(data['updated_at']!, _updatedAtMeta),
      );
    } else if (isInserting) {
      context.missing(_updatedAtMeta);
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  HealthMetric map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return HealthMetric(
      id:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}id'],
          )!,
      userId:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}user_id'],
          )!,
      logDate:
          attachedDatabase.typeMapping.read(
            DriftSqlType.dateTime,
            data['${effectivePrefix}log_date'],
          )!,
      steps:
          attachedDatabase.typeMapping.read(
            DriftSqlType.int,
            data['${effectivePrefix}steps'],
          )!,
      avgHeartRate: attachedDatabase.typeMapping.read(
        DriftSqlType.int,
        data['${effectivePrefix}avg_heart_rate'],
      ),
      sleepHours: attachedDatabase.typeMapping.read(
        DriftSqlType.double,
        data['${effectivePrefix}sleep_hours'],
      ),
      updatedAt:
          attachedDatabase.typeMapping.read(
            DriftSqlType.dateTime,
            data['${effectivePrefix}updated_at'],
          )!,
    );
  }

  @override
  $HealthMetricsTable createAlias(String alias) {
    return $HealthMetricsTable(attachedDatabase, alias);
  }
}

class HealthMetric extends DataClass implements Insertable<HealthMetric> {
  final String id;
  final String userId;
  final DateTime logDate;
  final int steps;
  final int? avgHeartRate;
  final double? sleepHours;
  final DateTime updatedAt;
  const HealthMetric({
    required this.id,
    required this.userId,
    required this.logDate,
    required this.steps,
    this.avgHeartRate,
    this.sleepHours,
    required this.updatedAt,
  });
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    map['user_id'] = Variable<String>(userId);
    map['log_date'] = Variable<DateTime>(logDate);
    map['steps'] = Variable<int>(steps);
    if (!nullToAbsent || avgHeartRate != null) {
      map['avg_heart_rate'] = Variable<int>(avgHeartRate);
    }
    if (!nullToAbsent || sleepHours != null) {
      map['sleep_hours'] = Variable<double>(sleepHours);
    }
    map['updated_at'] = Variable<DateTime>(updatedAt);
    return map;
  }

  HealthMetricsCompanion toCompanion(bool nullToAbsent) {
    return HealthMetricsCompanion(
      id: Value(id),
      userId: Value(userId),
      logDate: Value(logDate),
      steps: Value(steps),
      avgHeartRate:
          avgHeartRate == null && nullToAbsent
              ? const Value.absent()
              : Value(avgHeartRate),
      sleepHours:
          sleepHours == null && nullToAbsent
              ? const Value.absent()
              : Value(sleepHours),
      updatedAt: Value(updatedAt),
    );
  }

  factory HealthMetric.fromJson(
    Map<String, dynamic> json, {
    ValueSerializer? serializer,
  }) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return HealthMetric(
      id: serializer.fromJson<String>(json['id']),
      userId: serializer.fromJson<String>(json['userId']),
      logDate: serializer.fromJson<DateTime>(json['logDate']),
      steps: serializer.fromJson<int>(json['steps']),
      avgHeartRate: serializer.fromJson<int?>(json['avgHeartRate']),
      sleepHours: serializer.fromJson<double?>(json['sleepHours']),
      updatedAt: serializer.fromJson<DateTime>(json['updatedAt']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<String>(id),
      'userId': serializer.toJson<String>(userId),
      'logDate': serializer.toJson<DateTime>(logDate),
      'steps': serializer.toJson<int>(steps),
      'avgHeartRate': serializer.toJson<int?>(avgHeartRate),
      'sleepHours': serializer.toJson<double?>(sleepHours),
      'updatedAt': serializer.toJson<DateTime>(updatedAt),
    };
  }

  HealthMetric copyWith({
    String? id,
    String? userId,
    DateTime? logDate,
    int? steps,
    Value<int?> avgHeartRate = const Value.absent(),
    Value<double?> sleepHours = const Value.absent(),
    DateTime? updatedAt,
  }) => HealthMetric(
    id: id ?? this.id,
    userId: userId ?? this.userId,
    logDate: logDate ?? this.logDate,
    steps: steps ?? this.steps,
    avgHeartRate: avgHeartRate.present ? avgHeartRate.value : this.avgHeartRate,
    sleepHours: sleepHours.present ? sleepHours.value : this.sleepHours,
    updatedAt: updatedAt ?? this.updatedAt,
  );
  HealthMetric copyWithCompanion(HealthMetricsCompanion data) {
    return HealthMetric(
      id: data.id.present ? data.id.value : this.id,
      userId: data.userId.present ? data.userId.value : this.userId,
      logDate: data.logDate.present ? data.logDate.value : this.logDate,
      steps: data.steps.present ? data.steps.value : this.steps,
      avgHeartRate:
          data.avgHeartRate.present
              ? data.avgHeartRate.value
              : this.avgHeartRate,
      sleepHours:
          data.sleepHours.present ? data.sleepHours.value : this.sleepHours,
      updatedAt: data.updatedAt.present ? data.updatedAt.value : this.updatedAt,
    );
  }

  @override
  String toString() {
    return (StringBuffer('HealthMetric(')
          ..write('id: $id, ')
          ..write('userId: $userId, ')
          ..write('logDate: $logDate, ')
          ..write('steps: $steps, ')
          ..write('avgHeartRate: $avgHeartRate, ')
          ..write('sleepHours: $sleepHours, ')
          ..write('updatedAt: $updatedAt')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(
    id,
    userId,
    logDate,
    steps,
    avgHeartRate,
    sleepHours,
    updatedAt,
  );
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is HealthMetric &&
          other.id == this.id &&
          other.userId == this.userId &&
          other.logDate == this.logDate &&
          other.steps == this.steps &&
          other.avgHeartRate == this.avgHeartRate &&
          other.sleepHours == this.sleepHours &&
          other.updatedAt == this.updatedAt);
}

class HealthMetricsCompanion extends UpdateCompanion<HealthMetric> {
  final Value<String> id;
  final Value<String> userId;
  final Value<DateTime> logDate;
  final Value<int> steps;
  final Value<int?> avgHeartRate;
  final Value<double?> sleepHours;
  final Value<DateTime> updatedAt;
  final Value<int> rowid;
  const HealthMetricsCompanion({
    this.id = const Value.absent(),
    this.userId = const Value.absent(),
    this.logDate = const Value.absent(),
    this.steps = const Value.absent(),
    this.avgHeartRate = const Value.absent(),
    this.sleepHours = const Value.absent(),
    this.updatedAt = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  HealthMetricsCompanion.insert({
    required String id,
    required String userId,
    required DateTime logDate,
    this.steps = const Value.absent(),
    this.avgHeartRate = const Value.absent(),
    this.sleepHours = const Value.absent(),
    required DateTime updatedAt,
    this.rowid = const Value.absent(),
  }) : id = Value(id),
       userId = Value(userId),
       logDate = Value(logDate),
       updatedAt = Value(updatedAt);
  static Insertable<HealthMetric> custom({
    Expression<String>? id,
    Expression<String>? userId,
    Expression<DateTime>? logDate,
    Expression<int>? steps,
    Expression<int>? avgHeartRate,
    Expression<double>? sleepHours,
    Expression<DateTime>? updatedAt,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (userId != null) 'user_id': userId,
      if (logDate != null) 'log_date': logDate,
      if (steps != null) 'steps': steps,
      if (avgHeartRate != null) 'avg_heart_rate': avgHeartRate,
      if (sleepHours != null) 'sleep_hours': sleepHours,
      if (updatedAt != null) 'updated_at': updatedAt,
      if (rowid != null) 'rowid': rowid,
    });
  }

  HealthMetricsCompanion copyWith({
    Value<String>? id,
    Value<String>? userId,
    Value<DateTime>? logDate,
    Value<int>? steps,
    Value<int?>? avgHeartRate,
    Value<double?>? sleepHours,
    Value<DateTime>? updatedAt,
    Value<int>? rowid,
  }) {
    return HealthMetricsCompanion(
      id: id ?? this.id,
      userId: userId ?? this.userId,
      logDate: logDate ?? this.logDate,
      steps: steps ?? this.steps,
      avgHeartRate: avgHeartRate ?? this.avgHeartRate,
      sleepHours: sleepHours ?? this.sleepHours,
      updatedAt: updatedAt ?? this.updatedAt,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<String>(id.value);
    }
    if (userId.present) {
      map['user_id'] = Variable<String>(userId.value);
    }
    if (logDate.present) {
      map['log_date'] = Variable<DateTime>(logDate.value);
    }
    if (steps.present) {
      map['steps'] = Variable<int>(steps.value);
    }
    if (avgHeartRate.present) {
      map['avg_heart_rate'] = Variable<int>(avgHeartRate.value);
    }
    if (sleepHours.present) {
      map['sleep_hours'] = Variable<double>(sleepHours.value);
    }
    if (updatedAt.present) {
      map['updated_at'] = Variable<DateTime>(updatedAt.value);
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('HealthMetricsCompanion(')
          ..write('id: $id, ')
          ..write('userId: $userId, ')
          ..write('logDate: $logDate, ')
          ..write('steps: $steps, ')
          ..write('avgHeartRate: $avgHeartRate, ')
          ..write('sleepHours: $sleepHours, ')
          ..write('updatedAt: $updatedAt, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

class $PluginsTable extends Plugins with TableInfo<$PluginsTable, Plugin> {
  @override
  final GeneratedDatabase attachedDatabase;
  final String? _alias;
  $PluginsTable(this.attachedDatabase, [this._alias]);
  static const VerificationMeta _idMeta = const VerificationMeta('id');
  @override
  late final GeneratedColumn<String> id = GeneratedColumn<String>(
    'id',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _nameMeta = const VerificationMeta('name');
  @override
  late final GeneratedColumn<String> name = GeneratedColumn<String>(
    'name',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _slugMeta = const VerificationMeta('slug');
  @override
  late final GeneratedColumn<String> slug = GeneratedColumn<String>(
    'slug',
    aliasedName,
    false,
    type: DriftSqlType.string,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _descriptionMeta = const VerificationMeta(
    'description',
  );
  @override
  late final GeneratedColumn<String> description = GeneratedColumn<String>(
    'description',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _scriptCodeMeta = const VerificationMeta(
    'scriptCode',
  );
  @override
  late final GeneratedColumn<String> scriptCode = GeneratedColumn<String>(
    'script_code',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _versionMeta = const VerificationMeta(
    'version',
  );
  @override
  late final GeneratedColumn<String> version = GeneratedColumn<String>(
    'version',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _authorMeta = const VerificationMeta('author');
  @override
  late final GeneratedColumn<String> author = GeneratedColumn<String>(
    'author',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _intentsJsonMeta = const VerificationMeta(
    'intentsJson',
  );
  @override
  late final GeneratedColumn<String> intentsJson = GeneratedColumn<String>(
    'intents_json',
    aliasedName,
    true,
    type: DriftSqlType.string,
    requiredDuringInsert: false,
  );
  static const VerificationMeta _createdAtMeta = const VerificationMeta(
    'createdAt',
  );
  @override
  late final GeneratedColumn<DateTime> createdAt = GeneratedColumn<DateTime>(
    'created_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  static const VerificationMeta _updatedAtMeta = const VerificationMeta(
    'updatedAt',
  );
  @override
  late final GeneratedColumn<DateTime> updatedAt = GeneratedColumn<DateTime>(
    'updated_at',
    aliasedName,
    false,
    type: DriftSqlType.dateTime,
    requiredDuringInsert: true,
  );
  @override
  List<GeneratedColumn> get $columns => [
    id,
    name,
    slug,
    description,
    scriptCode,
    version,
    author,
    intentsJson,
    createdAt,
    updatedAt,
  ];
  @override
  String get aliasedName => _alias ?? actualTableName;
  @override
  String get actualTableName => $name;
  static const String $name = 'plugins';
  @override
  VerificationContext validateIntegrity(
    Insertable<Plugin> instance, {
    bool isInserting = false,
  }) {
    final context = VerificationContext();
    final data = instance.toColumns(true);
    if (data.containsKey('id')) {
      context.handle(_idMeta, id.isAcceptableOrUnknown(data['id']!, _idMeta));
    } else if (isInserting) {
      context.missing(_idMeta);
    }
    if (data.containsKey('name')) {
      context.handle(
        _nameMeta,
        name.isAcceptableOrUnknown(data['name']!, _nameMeta),
      );
    } else if (isInserting) {
      context.missing(_nameMeta);
    }
    if (data.containsKey('slug')) {
      context.handle(
        _slugMeta,
        slug.isAcceptableOrUnknown(data['slug']!, _slugMeta),
      );
    } else if (isInserting) {
      context.missing(_slugMeta);
    }
    if (data.containsKey('description')) {
      context.handle(
        _descriptionMeta,
        description.isAcceptableOrUnknown(
          data['description']!,
          _descriptionMeta,
        ),
      );
    }
    if (data.containsKey('script_code')) {
      context.handle(
        _scriptCodeMeta,
        scriptCode.isAcceptableOrUnknown(data['script_code']!, _scriptCodeMeta),
      );
    }
    if (data.containsKey('version')) {
      context.handle(
        _versionMeta,
        version.isAcceptableOrUnknown(data['version']!, _versionMeta),
      );
    }
    if (data.containsKey('author')) {
      context.handle(
        _authorMeta,
        author.isAcceptableOrUnknown(data['author']!, _authorMeta),
      );
    }
    if (data.containsKey('intents_json')) {
      context.handle(
        _intentsJsonMeta,
        intentsJson.isAcceptableOrUnknown(
          data['intents_json']!,
          _intentsJsonMeta,
        ),
      );
    }
    if (data.containsKey('created_at')) {
      context.handle(
        _createdAtMeta,
        createdAt.isAcceptableOrUnknown(data['created_at']!, _createdAtMeta),
      );
    } else if (isInserting) {
      context.missing(_createdAtMeta);
    }
    if (data.containsKey('updated_at')) {
      context.handle(
        _updatedAtMeta,
        updatedAt.isAcceptableOrUnknown(data['updated_at']!, _updatedAtMeta),
      );
    } else if (isInserting) {
      context.missing(_updatedAtMeta);
    }
    return context;
  }

  @override
  Set<GeneratedColumn> get $primaryKey => {id};
  @override
  Plugin map(Map<String, dynamic> data, {String? tablePrefix}) {
    final effectivePrefix = tablePrefix != null ? '$tablePrefix.' : '';
    return Plugin(
      id:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}id'],
          )!,
      name:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}name'],
          )!,
      slug:
          attachedDatabase.typeMapping.read(
            DriftSqlType.string,
            data['${effectivePrefix}slug'],
          )!,
      description: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}description'],
      ),
      scriptCode: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}script_code'],
      ),
      version: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}version'],
      ),
      author: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}author'],
      ),
      intentsJson: attachedDatabase.typeMapping.read(
        DriftSqlType.string,
        data['${effectivePrefix}intents_json'],
      ),
      createdAt:
          attachedDatabase.typeMapping.read(
            DriftSqlType.dateTime,
            data['${effectivePrefix}created_at'],
          )!,
      updatedAt:
          attachedDatabase.typeMapping.read(
            DriftSqlType.dateTime,
            data['${effectivePrefix}updated_at'],
          )!,
    );
  }

  @override
  $PluginsTable createAlias(String alias) {
    return $PluginsTable(attachedDatabase, alias);
  }
}

class Plugin extends DataClass implements Insertable<Plugin> {
  final String id;
  final String name;
  final String slug;
  final String? description;
  final String? scriptCode;
  final String? version;
  final String? author;
  final String? intentsJson;
  final DateTime createdAt;
  final DateTime updatedAt;
  const Plugin({
    required this.id,
    required this.name,
    required this.slug,
    this.description,
    this.scriptCode,
    this.version,
    this.author,
    this.intentsJson,
    required this.createdAt,
    required this.updatedAt,
  });
  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    map['id'] = Variable<String>(id);
    map['name'] = Variable<String>(name);
    map['slug'] = Variable<String>(slug);
    if (!nullToAbsent || description != null) {
      map['description'] = Variable<String>(description);
    }
    if (!nullToAbsent || scriptCode != null) {
      map['script_code'] = Variable<String>(scriptCode);
    }
    if (!nullToAbsent || version != null) {
      map['version'] = Variable<String>(version);
    }
    if (!nullToAbsent || author != null) {
      map['author'] = Variable<String>(author);
    }
    if (!nullToAbsent || intentsJson != null) {
      map['intents_json'] = Variable<String>(intentsJson);
    }
    map['created_at'] = Variable<DateTime>(createdAt);
    map['updated_at'] = Variable<DateTime>(updatedAt);
    return map;
  }

  PluginsCompanion toCompanion(bool nullToAbsent) {
    return PluginsCompanion(
      id: Value(id),
      name: Value(name),
      slug: Value(slug),
      description:
          description == null && nullToAbsent
              ? const Value.absent()
              : Value(description),
      scriptCode:
          scriptCode == null && nullToAbsent
              ? const Value.absent()
              : Value(scriptCode),
      version:
          version == null && nullToAbsent
              ? const Value.absent()
              : Value(version),
      author:
          author == null && nullToAbsent ? const Value.absent() : Value(author),
      intentsJson:
          intentsJson == null && nullToAbsent
              ? const Value.absent()
              : Value(intentsJson),
      createdAt: Value(createdAt),
      updatedAt: Value(updatedAt),
    );
  }

  factory Plugin.fromJson(
    Map<String, dynamic> json, {
    ValueSerializer? serializer,
  }) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return Plugin(
      id: serializer.fromJson<String>(json['id']),
      name: serializer.fromJson<String>(json['name']),
      slug: serializer.fromJson<String>(json['slug']),
      description: serializer.fromJson<String?>(json['description']),
      scriptCode: serializer.fromJson<String?>(json['scriptCode']),
      version: serializer.fromJson<String?>(json['version']),
      author: serializer.fromJson<String?>(json['author']),
      intentsJson: serializer.fromJson<String?>(json['intentsJson']),
      createdAt: serializer.fromJson<DateTime>(json['createdAt']),
      updatedAt: serializer.fromJson<DateTime>(json['updatedAt']),
    );
  }
  @override
  Map<String, dynamic> toJson({ValueSerializer? serializer}) {
    serializer ??= driftRuntimeOptions.defaultSerializer;
    return <String, dynamic>{
      'id': serializer.toJson<String>(id),
      'name': serializer.toJson<String>(name),
      'slug': serializer.toJson<String>(slug),
      'description': serializer.toJson<String?>(description),
      'scriptCode': serializer.toJson<String?>(scriptCode),
      'version': serializer.toJson<String?>(version),
      'author': serializer.toJson<String?>(author),
      'intentsJson': serializer.toJson<String?>(intentsJson),
      'createdAt': serializer.toJson<DateTime>(createdAt),
      'updatedAt': serializer.toJson<DateTime>(updatedAt),
    };
  }

  Plugin copyWith({
    String? id,
    String? name,
    String? slug,
    Value<String?> description = const Value.absent(),
    Value<String?> scriptCode = const Value.absent(),
    Value<String?> version = const Value.absent(),
    Value<String?> author = const Value.absent(),
    Value<String?> intentsJson = const Value.absent(),
    DateTime? createdAt,
    DateTime? updatedAt,
  }) => Plugin(
    id: id ?? this.id,
    name: name ?? this.name,
    slug: slug ?? this.slug,
    description: description.present ? description.value : this.description,
    scriptCode: scriptCode.present ? scriptCode.value : this.scriptCode,
    version: version.present ? version.value : this.version,
    author: author.present ? author.value : this.author,
    intentsJson: intentsJson.present ? intentsJson.value : this.intentsJson,
    createdAt: createdAt ?? this.createdAt,
    updatedAt: updatedAt ?? this.updatedAt,
  );
  Plugin copyWithCompanion(PluginsCompanion data) {
    return Plugin(
      id: data.id.present ? data.id.value : this.id,
      name: data.name.present ? data.name.value : this.name,
      slug: data.slug.present ? data.slug.value : this.slug,
      description:
          data.description.present ? data.description.value : this.description,
      scriptCode:
          data.scriptCode.present ? data.scriptCode.value : this.scriptCode,
      version: data.version.present ? data.version.value : this.version,
      author: data.author.present ? data.author.value : this.author,
      intentsJson:
          data.intentsJson.present ? data.intentsJson.value : this.intentsJson,
      createdAt: data.createdAt.present ? data.createdAt.value : this.createdAt,
      updatedAt: data.updatedAt.present ? data.updatedAt.value : this.updatedAt,
    );
  }

  @override
  String toString() {
    return (StringBuffer('Plugin(')
          ..write('id: $id, ')
          ..write('name: $name, ')
          ..write('slug: $slug, ')
          ..write('description: $description, ')
          ..write('scriptCode: $scriptCode, ')
          ..write('version: $version, ')
          ..write('author: $author, ')
          ..write('intentsJson: $intentsJson, ')
          ..write('createdAt: $createdAt, ')
          ..write('updatedAt: $updatedAt')
          ..write(')'))
        .toString();
  }

  @override
  int get hashCode => Object.hash(
    id,
    name,
    slug,
    description,
    scriptCode,
    version,
    author,
    intentsJson,
    createdAt,
    updatedAt,
  );
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is Plugin &&
          other.id == this.id &&
          other.name == this.name &&
          other.slug == this.slug &&
          other.description == this.description &&
          other.scriptCode == this.scriptCode &&
          other.version == this.version &&
          other.author == this.author &&
          other.intentsJson == this.intentsJson &&
          other.createdAt == this.createdAt &&
          other.updatedAt == this.updatedAt);
}

class PluginsCompanion extends UpdateCompanion<Plugin> {
  final Value<String> id;
  final Value<String> name;
  final Value<String> slug;
  final Value<String?> description;
  final Value<String?> scriptCode;
  final Value<String?> version;
  final Value<String?> author;
  final Value<String?> intentsJson;
  final Value<DateTime> createdAt;
  final Value<DateTime> updatedAt;
  final Value<int> rowid;
  const PluginsCompanion({
    this.id = const Value.absent(),
    this.name = const Value.absent(),
    this.slug = const Value.absent(),
    this.description = const Value.absent(),
    this.scriptCode = const Value.absent(),
    this.version = const Value.absent(),
    this.author = const Value.absent(),
    this.intentsJson = const Value.absent(),
    this.createdAt = const Value.absent(),
    this.updatedAt = const Value.absent(),
    this.rowid = const Value.absent(),
  });
  PluginsCompanion.insert({
    required String id,
    required String name,
    required String slug,
    this.description = const Value.absent(),
    this.scriptCode = const Value.absent(),
    this.version = const Value.absent(),
    this.author = const Value.absent(),
    this.intentsJson = const Value.absent(),
    required DateTime createdAt,
    required DateTime updatedAt,
    this.rowid = const Value.absent(),
  }) : id = Value(id),
       name = Value(name),
       slug = Value(slug),
       createdAt = Value(createdAt),
       updatedAt = Value(updatedAt);
  static Insertable<Plugin> custom({
    Expression<String>? id,
    Expression<String>? name,
    Expression<String>? slug,
    Expression<String>? description,
    Expression<String>? scriptCode,
    Expression<String>? version,
    Expression<String>? author,
    Expression<String>? intentsJson,
    Expression<DateTime>? createdAt,
    Expression<DateTime>? updatedAt,
    Expression<int>? rowid,
  }) {
    return RawValuesInsertable({
      if (id != null) 'id': id,
      if (name != null) 'name': name,
      if (slug != null) 'slug': slug,
      if (description != null) 'description': description,
      if (scriptCode != null) 'script_code': scriptCode,
      if (version != null) 'version': version,
      if (author != null) 'author': author,
      if (intentsJson != null) 'intents_json': intentsJson,
      if (createdAt != null) 'created_at': createdAt,
      if (updatedAt != null) 'updated_at': updatedAt,
      if (rowid != null) 'rowid': rowid,
    });
  }

  PluginsCompanion copyWith({
    Value<String>? id,
    Value<String>? name,
    Value<String>? slug,
    Value<String?>? description,
    Value<String?>? scriptCode,
    Value<String?>? version,
    Value<String?>? author,
    Value<String?>? intentsJson,
    Value<DateTime>? createdAt,
    Value<DateTime>? updatedAt,
    Value<int>? rowid,
  }) {
    return PluginsCompanion(
      id: id ?? this.id,
      name: name ?? this.name,
      slug: slug ?? this.slug,
      description: description ?? this.description,
      scriptCode: scriptCode ?? this.scriptCode,
      version: version ?? this.version,
      author: author ?? this.author,
      intentsJson: intentsJson ?? this.intentsJson,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
      rowid: rowid ?? this.rowid,
    );
  }

  @override
  Map<String, Expression> toColumns(bool nullToAbsent) {
    final map = <String, Expression>{};
    if (id.present) {
      map['id'] = Variable<String>(id.value);
    }
    if (name.present) {
      map['name'] = Variable<String>(name.value);
    }
    if (slug.present) {
      map['slug'] = Variable<String>(slug.value);
    }
    if (description.present) {
      map['description'] = Variable<String>(description.value);
    }
    if (scriptCode.present) {
      map['script_code'] = Variable<String>(scriptCode.value);
    }
    if (version.present) {
      map['version'] = Variable<String>(version.value);
    }
    if (author.present) {
      map['author'] = Variable<String>(author.value);
    }
    if (intentsJson.present) {
      map['intents_json'] = Variable<String>(intentsJson.value);
    }
    if (createdAt.present) {
      map['created_at'] = Variable<DateTime>(createdAt.value);
    }
    if (updatedAt.present) {
      map['updated_at'] = Variable<DateTime>(updatedAt.value);
    }
    if (rowid.present) {
      map['rowid'] = Variable<int>(rowid.value);
    }
    return map;
  }

  @override
  String toString() {
    return (StringBuffer('PluginsCompanion(')
          ..write('id: $id, ')
          ..write('name: $name, ')
          ..write('slug: $slug, ')
          ..write('description: $description, ')
          ..write('scriptCode: $scriptCode, ')
          ..write('version: $version, ')
          ..write('author: $author, ')
          ..write('intentsJson: $intentsJson, ')
          ..write('createdAt: $createdAt, ')
          ..write('updatedAt: $updatedAt, ')
          ..write('rowid: $rowid')
          ..write(')'))
        .toString();
  }
}

abstract class _$NomiDatabase extends GeneratedDatabase {
  _$NomiDatabase(QueryExecutor e) : super(e);
  $NomiDatabaseManager get managers => $NomiDatabaseManager(this);
  late final $ConversationsTable conversations = $ConversationsTable(this);
  late final $MessagesTable messages = $MessagesTable(this);
  late final $RemindersTable reminders = $RemindersTable(this);
  late final $TransactionsTable transactions = $TransactionsTable(this);
  late final $HealthMetricsTable healthMetrics = $HealthMetricsTable(this);
  late final $PluginsTable plugins = $PluginsTable(this);
  @override
  Iterable<TableInfo<Table, Object?>> get allTables =>
      allSchemaEntities.whereType<TableInfo<Table, Object?>>();
  @override
  List<DatabaseSchemaEntity> get allSchemaEntities => [
    conversations,
    messages,
    reminders,
    transactions,
    healthMetrics,
    plugins,
  ];
}

typedef $$ConversationsTableCreateCompanionBuilder =
    ConversationsCompanion Function({
      required String id,
      Value<String?> name,
      Value<int?> cumulativeTokens,
      Value<int?> maxTokenUsage,
      required DateTime createdAt,
      required DateTime updatedAt,
      Value<int> rowid,
    });
typedef $$ConversationsTableUpdateCompanionBuilder =
    ConversationsCompanion Function({
      Value<String> id,
      Value<String?> name,
      Value<int?> cumulativeTokens,
      Value<int?> maxTokenUsage,
      Value<DateTime> createdAt,
      Value<DateTime> updatedAt,
      Value<int> rowid,
    });

final class $$ConversationsTableReferences
    extends BaseReferences<_$NomiDatabase, $ConversationsTable, Conversation> {
  $$ConversationsTableReferences(
    super.$_db,
    super.$_table,
    super.$_typedResult,
  );

  static MultiTypedResultKey<$MessagesTable, List<Message>> _messagesRefsTable(
    _$NomiDatabase db,
  ) => MultiTypedResultKey.fromTable(
    db.messages,
    aliasName: $_aliasNameGenerator(
      db.conversations.id,
      db.messages.conversationId,
    ),
  );

  $$MessagesTableProcessedTableManager get messagesRefs {
    final manager = $$MessagesTableTableManager(
      $_db,
      $_db.messages,
    ).filter((f) => f.conversationId.id.sqlEquals($_itemColumn<String>('id')!));

    final cache = $_typedResult.readTableOrNull(_messagesRefsTable($_db));
    return ProcessedTableManager(
      manager.$state.copyWith(prefetchedData: cache),
    );
  }
}

class $$ConversationsTableFilterComposer
    extends Composer<_$NomiDatabase, $ConversationsTable> {
  $$ConversationsTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get name => $composableBuilder(
    column: $table.name,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<int> get cumulativeTokens => $composableBuilder(
    column: $table.cumulativeTokens,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<int> get maxTokenUsage => $composableBuilder(
    column: $table.maxTokenUsage,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get updatedAt => $composableBuilder(
    column: $table.updatedAt,
    builder: (column) => ColumnFilters(column),
  );

  Expression<bool> messagesRefs(
    Expression<bool> Function($$MessagesTableFilterComposer f) f,
  ) {
    final $$MessagesTableFilterComposer composer = $composerBuilder(
      composer: this,
      getCurrentColumn: (t) => t.id,
      referencedTable: $db.messages,
      getReferencedColumn: (t) => t.conversationId,
      builder:
          (
            joinBuilder, {
            $addJoinBuilderToRootComposer,
            $removeJoinBuilderFromRootComposer,
          }) => $$MessagesTableFilterComposer(
            $db: $db,
            $table: $db.messages,
            $addJoinBuilderToRootComposer: $addJoinBuilderToRootComposer,
            joinBuilder: joinBuilder,
            $removeJoinBuilderFromRootComposer:
                $removeJoinBuilderFromRootComposer,
          ),
    );
    return f(composer);
  }
}

class $$ConversationsTableOrderingComposer
    extends Composer<_$NomiDatabase, $ConversationsTable> {
  $$ConversationsTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get name => $composableBuilder(
    column: $table.name,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get cumulativeTokens => $composableBuilder(
    column: $table.cumulativeTokens,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get maxTokenUsage => $composableBuilder(
    column: $table.maxTokenUsage,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get updatedAt => $composableBuilder(
    column: $table.updatedAt,
    builder: (column) => ColumnOrderings(column),
  );
}

class $$ConversationsTableAnnotationComposer
    extends Composer<_$NomiDatabase, $ConversationsTable> {
  $$ConversationsTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<String> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get name =>
      $composableBuilder(column: $table.name, builder: (column) => column);

  GeneratedColumn<int> get cumulativeTokens => $composableBuilder(
    column: $table.cumulativeTokens,
    builder: (column) => column,
  );

  GeneratedColumn<int> get maxTokenUsage => $composableBuilder(
    column: $table.maxTokenUsage,
    builder: (column) => column,
  );

  GeneratedColumn<DateTime> get createdAt =>
      $composableBuilder(column: $table.createdAt, builder: (column) => column);

  GeneratedColumn<DateTime> get updatedAt =>
      $composableBuilder(column: $table.updatedAt, builder: (column) => column);

  Expression<T> messagesRefs<T extends Object>(
    Expression<T> Function($$MessagesTableAnnotationComposer a) f,
  ) {
    final $$MessagesTableAnnotationComposer composer = $composerBuilder(
      composer: this,
      getCurrentColumn: (t) => t.id,
      referencedTable: $db.messages,
      getReferencedColumn: (t) => t.conversationId,
      builder:
          (
            joinBuilder, {
            $addJoinBuilderToRootComposer,
            $removeJoinBuilderFromRootComposer,
          }) => $$MessagesTableAnnotationComposer(
            $db: $db,
            $table: $db.messages,
            $addJoinBuilderToRootComposer: $addJoinBuilderToRootComposer,
            joinBuilder: joinBuilder,
            $removeJoinBuilderFromRootComposer:
                $removeJoinBuilderFromRootComposer,
          ),
    );
    return f(composer);
  }
}

class $$ConversationsTableTableManager
    extends
        RootTableManager<
          _$NomiDatabase,
          $ConversationsTable,
          Conversation,
          $$ConversationsTableFilterComposer,
          $$ConversationsTableOrderingComposer,
          $$ConversationsTableAnnotationComposer,
          $$ConversationsTableCreateCompanionBuilder,
          $$ConversationsTableUpdateCompanionBuilder,
          (Conversation, $$ConversationsTableReferences),
          Conversation,
          PrefetchHooks Function({bool messagesRefs})
        > {
  $$ConversationsTableTableManager(_$NomiDatabase db, $ConversationsTable table)
    : super(
        TableManagerState(
          db: db,
          table: table,
          createFilteringComposer:
              () => $$ConversationsTableFilterComposer($db: db, $table: table),
          createOrderingComposer:
              () =>
                  $$ConversationsTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer:
              () => $$ConversationsTableAnnotationComposer(
                $db: db,
                $table: table,
              ),
          updateCompanionCallback:
              ({
                Value<String> id = const Value.absent(),
                Value<String?> name = const Value.absent(),
                Value<int?> cumulativeTokens = const Value.absent(),
                Value<int?> maxTokenUsage = const Value.absent(),
                Value<DateTime> createdAt = const Value.absent(),
                Value<DateTime> updatedAt = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => ConversationsCompanion(
                id: id,
                name: name,
                cumulativeTokens: cumulativeTokens,
                maxTokenUsage: maxTokenUsage,
                createdAt: createdAt,
                updatedAt: updatedAt,
                rowid: rowid,
              ),
          createCompanionCallback:
              ({
                required String id,
                Value<String?> name = const Value.absent(),
                Value<int?> cumulativeTokens = const Value.absent(),
                Value<int?> maxTokenUsage = const Value.absent(),
                required DateTime createdAt,
                required DateTime updatedAt,
                Value<int> rowid = const Value.absent(),
              }) => ConversationsCompanion.insert(
                id: id,
                name: name,
                cumulativeTokens: cumulativeTokens,
                maxTokenUsage: maxTokenUsage,
                createdAt: createdAt,
                updatedAt: updatedAt,
                rowid: rowid,
              ),
          withReferenceMapper:
              (p0) =>
                  p0
                      .map(
                        (e) => (
                          e.readTable(table),
                          $$ConversationsTableReferences(db, table, e),
                        ),
                      )
                      .toList(),
          prefetchHooksCallback: ({messagesRefs = false}) {
            return PrefetchHooks(
              db: db,
              explicitlyWatchedTables: [if (messagesRefs) db.messages],
              addJoins: null,
              getPrefetchedDataCallback: (items) async {
                return [
                  if (messagesRefs)
                    await $_getPrefetchedData<
                      Conversation,
                      $ConversationsTable,
                      Message
                    >(
                      currentTable: table,
                      referencedTable: $$ConversationsTableReferences
                          ._messagesRefsTable(db),
                      managerFromTypedResult:
                          (p0) =>
                              $$ConversationsTableReferences(
                                db,
                                table,
                                p0,
                              ).messagesRefs,
                      referencedItemsForCurrentItem:
                          (item, referencedItems) => referencedItems.where(
                            (e) => e.conversationId == item.id,
                          ),
                      typedResults: items,
                    ),
                ];
              },
            );
          },
        ),
      );
}

typedef $$ConversationsTableProcessedTableManager =
    ProcessedTableManager<
      _$NomiDatabase,
      $ConversationsTable,
      Conversation,
      $$ConversationsTableFilterComposer,
      $$ConversationsTableOrderingComposer,
      $$ConversationsTableAnnotationComposer,
      $$ConversationsTableCreateCompanionBuilder,
      $$ConversationsTableUpdateCompanionBuilder,
      (Conversation, $$ConversationsTableReferences),
      Conversation,
      PrefetchHooks Function({bool messagesRefs})
    >;
typedef $$MessagesTableCreateCompanionBuilder =
    MessagesCompanion Function({
      required String id,
      required String conversationId,
      required String role,
      required String content,
      Value<String?> displayName,
      Value<String?> thought,
      Value<String?> imageUrl,
      Value<String?> videoUrl,
      Value<String?> audioUrl,
      Value<String?> documentUrl,
      Value<String?> stickerUrl,
      Value<String?> userId,
      Value<int?> totalTokens,
      required DateTime createdAt,
      Value<String?> metadata,
      Value<String?> replyToId,
      Value<String?> repliedMessage,
      required SyncStatus syncStatus,
      Value<int> rowid,
    });
typedef $$MessagesTableUpdateCompanionBuilder =
    MessagesCompanion Function({
      Value<String> id,
      Value<String> conversationId,
      Value<String> role,
      Value<String> content,
      Value<String?> displayName,
      Value<String?> thought,
      Value<String?> imageUrl,
      Value<String?> videoUrl,
      Value<String?> audioUrl,
      Value<String?> documentUrl,
      Value<String?> stickerUrl,
      Value<String?> userId,
      Value<int?> totalTokens,
      Value<DateTime> createdAt,
      Value<String?> metadata,
      Value<String?> replyToId,
      Value<String?> repliedMessage,
      Value<SyncStatus> syncStatus,
      Value<int> rowid,
    });

final class $$MessagesTableReferences
    extends BaseReferences<_$NomiDatabase, $MessagesTable, Message> {
  $$MessagesTableReferences(super.$_db, super.$_table, super.$_typedResult);

  static $ConversationsTable _conversationIdTable(_$NomiDatabase db) =>
      db.conversations.createAlias(
        $_aliasNameGenerator(db.messages.conversationId, db.conversations.id),
      );

  $$ConversationsTableProcessedTableManager get conversationId {
    final $_column = $_itemColumn<String>('conversation_id')!;

    final manager = $$ConversationsTableTableManager(
      $_db,
      $_db.conversations,
    ).filter((f) => f.id.sqlEquals($_column));
    final item = $_typedResult.readTableOrNull(_conversationIdTable($_db));
    if (item == null) return manager;
    return ProcessedTableManager(
      manager.$state.copyWith(prefetchedData: [item]),
    );
  }
}

class $$MessagesTableFilterComposer
    extends Composer<_$NomiDatabase, $MessagesTable> {
  $$MessagesTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get role => $composableBuilder(
    column: $table.role,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get content => $composableBuilder(
    column: $table.content,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get displayName => $composableBuilder(
    column: $table.displayName,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get thought => $composableBuilder(
    column: $table.thought,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get imageUrl => $composableBuilder(
    column: $table.imageUrl,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get videoUrl => $composableBuilder(
    column: $table.videoUrl,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get audioUrl => $composableBuilder(
    column: $table.audioUrl,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get documentUrl => $composableBuilder(
    column: $table.documentUrl,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get stickerUrl => $composableBuilder(
    column: $table.stickerUrl,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get userId => $composableBuilder(
    column: $table.userId,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<int> get totalTokens => $composableBuilder(
    column: $table.totalTokens,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get metadata => $composableBuilder(
    column: $table.metadata,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get replyToId => $composableBuilder(
    column: $table.replyToId,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get repliedMessage => $composableBuilder(
    column: $table.repliedMessage,
    builder: (column) => ColumnFilters(column),
  );

  ColumnWithTypeConverterFilters<SyncStatus, SyncStatus, int> get syncStatus =>
      $composableBuilder(
        column: $table.syncStatus,
        builder: (column) => ColumnWithTypeConverterFilters(column),
      );

  $$ConversationsTableFilterComposer get conversationId {
    final $$ConversationsTableFilterComposer composer = $composerBuilder(
      composer: this,
      getCurrentColumn: (t) => t.conversationId,
      referencedTable: $db.conversations,
      getReferencedColumn: (t) => t.id,
      builder:
          (
            joinBuilder, {
            $addJoinBuilderToRootComposer,
            $removeJoinBuilderFromRootComposer,
          }) => $$ConversationsTableFilterComposer(
            $db: $db,
            $table: $db.conversations,
            $addJoinBuilderToRootComposer: $addJoinBuilderToRootComposer,
            joinBuilder: joinBuilder,
            $removeJoinBuilderFromRootComposer:
                $removeJoinBuilderFromRootComposer,
          ),
    );
    return composer;
  }
}

class $$MessagesTableOrderingComposer
    extends Composer<_$NomiDatabase, $MessagesTable> {
  $$MessagesTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get role => $composableBuilder(
    column: $table.role,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get content => $composableBuilder(
    column: $table.content,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get displayName => $composableBuilder(
    column: $table.displayName,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get thought => $composableBuilder(
    column: $table.thought,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get imageUrl => $composableBuilder(
    column: $table.imageUrl,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get videoUrl => $composableBuilder(
    column: $table.videoUrl,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get audioUrl => $composableBuilder(
    column: $table.audioUrl,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get documentUrl => $composableBuilder(
    column: $table.documentUrl,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get stickerUrl => $composableBuilder(
    column: $table.stickerUrl,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get userId => $composableBuilder(
    column: $table.userId,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get totalTokens => $composableBuilder(
    column: $table.totalTokens,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get metadata => $composableBuilder(
    column: $table.metadata,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get replyToId => $composableBuilder(
    column: $table.replyToId,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get repliedMessage => $composableBuilder(
    column: $table.repliedMessage,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get syncStatus => $composableBuilder(
    column: $table.syncStatus,
    builder: (column) => ColumnOrderings(column),
  );

  $$ConversationsTableOrderingComposer get conversationId {
    final $$ConversationsTableOrderingComposer composer = $composerBuilder(
      composer: this,
      getCurrentColumn: (t) => t.conversationId,
      referencedTable: $db.conversations,
      getReferencedColumn: (t) => t.id,
      builder:
          (
            joinBuilder, {
            $addJoinBuilderToRootComposer,
            $removeJoinBuilderFromRootComposer,
          }) => $$ConversationsTableOrderingComposer(
            $db: $db,
            $table: $db.conversations,
            $addJoinBuilderToRootComposer: $addJoinBuilderToRootComposer,
            joinBuilder: joinBuilder,
            $removeJoinBuilderFromRootComposer:
                $removeJoinBuilderFromRootComposer,
          ),
    );
    return composer;
  }
}

class $$MessagesTableAnnotationComposer
    extends Composer<_$NomiDatabase, $MessagesTable> {
  $$MessagesTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<String> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get role =>
      $composableBuilder(column: $table.role, builder: (column) => column);

  GeneratedColumn<String> get content =>
      $composableBuilder(column: $table.content, builder: (column) => column);

  GeneratedColumn<String> get displayName => $composableBuilder(
    column: $table.displayName,
    builder: (column) => column,
  );

  GeneratedColumn<String> get thought =>
      $composableBuilder(column: $table.thought, builder: (column) => column);

  GeneratedColumn<String> get imageUrl =>
      $composableBuilder(column: $table.imageUrl, builder: (column) => column);

  GeneratedColumn<String> get videoUrl =>
      $composableBuilder(column: $table.videoUrl, builder: (column) => column);

  GeneratedColumn<String> get audioUrl =>
      $composableBuilder(column: $table.audioUrl, builder: (column) => column);

  GeneratedColumn<String> get documentUrl => $composableBuilder(
    column: $table.documentUrl,
    builder: (column) => column,
  );

  GeneratedColumn<String> get stickerUrl => $composableBuilder(
    column: $table.stickerUrl,
    builder: (column) => column,
  );

  GeneratedColumn<String> get userId =>
      $composableBuilder(column: $table.userId, builder: (column) => column);

  GeneratedColumn<int> get totalTokens => $composableBuilder(
    column: $table.totalTokens,
    builder: (column) => column,
  );

  GeneratedColumn<DateTime> get createdAt =>
      $composableBuilder(column: $table.createdAt, builder: (column) => column);

  GeneratedColumn<String> get metadata =>
      $composableBuilder(column: $table.metadata, builder: (column) => column);

  GeneratedColumn<String> get replyToId =>
      $composableBuilder(column: $table.replyToId, builder: (column) => column);

  GeneratedColumn<String> get repliedMessage => $composableBuilder(
    column: $table.repliedMessage,
    builder: (column) => column,
  );

  GeneratedColumnWithTypeConverter<SyncStatus, int> get syncStatus =>
      $composableBuilder(
        column: $table.syncStatus,
        builder: (column) => column,
      );

  $$ConversationsTableAnnotationComposer get conversationId {
    final $$ConversationsTableAnnotationComposer composer = $composerBuilder(
      composer: this,
      getCurrentColumn: (t) => t.conversationId,
      referencedTable: $db.conversations,
      getReferencedColumn: (t) => t.id,
      builder:
          (
            joinBuilder, {
            $addJoinBuilderToRootComposer,
            $removeJoinBuilderFromRootComposer,
          }) => $$ConversationsTableAnnotationComposer(
            $db: $db,
            $table: $db.conversations,
            $addJoinBuilderToRootComposer: $addJoinBuilderToRootComposer,
            joinBuilder: joinBuilder,
            $removeJoinBuilderFromRootComposer:
                $removeJoinBuilderFromRootComposer,
          ),
    );
    return composer;
  }
}

class $$MessagesTableTableManager
    extends
        RootTableManager<
          _$NomiDatabase,
          $MessagesTable,
          Message,
          $$MessagesTableFilterComposer,
          $$MessagesTableOrderingComposer,
          $$MessagesTableAnnotationComposer,
          $$MessagesTableCreateCompanionBuilder,
          $$MessagesTableUpdateCompanionBuilder,
          (Message, $$MessagesTableReferences),
          Message,
          PrefetchHooks Function({bool conversationId})
        > {
  $$MessagesTableTableManager(_$NomiDatabase db, $MessagesTable table)
    : super(
        TableManagerState(
          db: db,
          table: table,
          createFilteringComposer:
              () => $$MessagesTableFilterComposer($db: db, $table: table),
          createOrderingComposer:
              () => $$MessagesTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer:
              () => $$MessagesTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback:
              ({
                Value<String> id = const Value.absent(),
                Value<String> conversationId = const Value.absent(),
                Value<String> role = const Value.absent(),
                Value<String> content = const Value.absent(),
                Value<String?> displayName = const Value.absent(),
                Value<String?> thought = const Value.absent(),
                Value<String?> imageUrl = const Value.absent(),
                Value<String?> videoUrl = const Value.absent(),
                Value<String?> audioUrl = const Value.absent(),
                Value<String?> documentUrl = const Value.absent(),
                Value<String?> stickerUrl = const Value.absent(),
                Value<String?> userId = const Value.absent(),
                Value<int?> totalTokens = const Value.absent(),
                Value<DateTime> createdAt = const Value.absent(),
                Value<String?> metadata = const Value.absent(),
                Value<String?> replyToId = const Value.absent(),
                Value<String?> repliedMessage = const Value.absent(),
                Value<SyncStatus> syncStatus = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => MessagesCompanion(
                id: id,
                conversationId: conversationId,
                role: role,
                content: content,
                displayName: displayName,
                thought: thought,
                imageUrl: imageUrl,
                videoUrl: videoUrl,
                audioUrl: audioUrl,
                documentUrl: documentUrl,
                stickerUrl: stickerUrl,
                userId: userId,
                totalTokens: totalTokens,
                createdAt: createdAt,
                metadata: metadata,
                replyToId: replyToId,
                repliedMessage: repliedMessage,
                syncStatus: syncStatus,
                rowid: rowid,
              ),
          createCompanionCallback:
              ({
                required String id,
                required String conversationId,
                required String role,
                required String content,
                Value<String?> displayName = const Value.absent(),
                Value<String?> thought = const Value.absent(),
                Value<String?> imageUrl = const Value.absent(),
                Value<String?> videoUrl = const Value.absent(),
                Value<String?> audioUrl = const Value.absent(),
                Value<String?> documentUrl = const Value.absent(),
                Value<String?> stickerUrl = const Value.absent(),
                Value<String?> userId = const Value.absent(),
                Value<int?> totalTokens = const Value.absent(),
                required DateTime createdAt,
                Value<String?> metadata = const Value.absent(),
                Value<String?> replyToId = const Value.absent(),
                Value<String?> repliedMessage = const Value.absent(),
                required SyncStatus syncStatus,
                Value<int> rowid = const Value.absent(),
              }) => MessagesCompanion.insert(
                id: id,
                conversationId: conversationId,
                role: role,
                content: content,
                displayName: displayName,
                thought: thought,
                imageUrl: imageUrl,
                videoUrl: videoUrl,
                audioUrl: audioUrl,
                documentUrl: documentUrl,
                stickerUrl: stickerUrl,
                userId: userId,
                totalTokens: totalTokens,
                createdAt: createdAt,
                metadata: metadata,
                replyToId: replyToId,
                repliedMessage: repliedMessage,
                syncStatus: syncStatus,
                rowid: rowid,
              ),
          withReferenceMapper:
              (p0) =>
                  p0
                      .map(
                        (e) => (
                          e.readTable(table),
                          $$MessagesTableReferences(db, table, e),
                        ),
                      )
                      .toList(),
          prefetchHooksCallback: ({conversationId = false}) {
            return PrefetchHooks(
              db: db,
              explicitlyWatchedTables: [],
              addJoins: <
                T extends TableManagerState<
                  dynamic,
                  dynamic,
                  dynamic,
                  dynamic,
                  dynamic,
                  dynamic,
                  dynamic,
                  dynamic,
                  dynamic,
                  dynamic,
                  dynamic
                >
              >(state) {
                if (conversationId) {
                  state =
                      state.withJoin(
                            currentTable: table,
                            currentColumn: table.conversationId,
                            referencedTable: $$MessagesTableReferences
                                ._conversationIdTable(db),
                            referencedColumn:
                                $$MessagesTableReferences
                                    ._conversationIdTable(db)
                                    .id,
                          )
                          as T;
                }

                return state;
              },
              getPrefetchedDataCallback: (items) async {
                return [];
              },
            );
          },
        ),
      );
}

typedef $$MessagesTableProcessedTableManager =
    ProcessedTableManager<
      _$NomiDatabase,
      $MessagesTable,
      Message,
      $$MessagesTableFilterComposer,
      $$MessagesTableOrderingComposer,
      $$MessagesTableAnnotationComposer,
      $$MessagesTableCreateCompanionBuilder,
      $$MessagesTableUpdateCompanionBuilder,
      (Message, $$MessagesTableReferences),
      Message,
      PrefetchHooks Function({bool conversationId})
    >;
typedef $$RemindersTableCreateCompanionBuilder =
    RemindersCompanion Function({
      required String id,
      required String content,
      Value<String?> taskType,
      Value<String?> frequency,
      required String status,
      required DateTime dueAt,
      required DateTime createdAt,
      Value<String?> userDisplayName,
      Value<String?> conversationTitle,
      Value<int> rowid,
    });
typedef $$RemindersTableUpdateCompanionBuilder =
    RemindersCompanion Function({
      Value<String> id,
      Value<String> content,
      Value<String?> taskType,
      Value<String?> frequency,
      Value<String> status,
      Value<DateTime> dueAt,
      Value<DateTime> createdAt,
      Value<String?> userDisplayName,
      Value<String?> conversationTitle,
      Value<int> rowid,
    });

class $$RemindersTableFilterComposer
    extends Composer<_$NomiDatabase, $RemindersTable> {
  $$RemindersTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get content => $composableBuilder(
    column: $table.content,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get taskType => $composableBuilder(
    column: $table.taskType,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get frequency => $composableBuilder(
    column: $table.frequency,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get status => $composableBuilder(
    column: $table.status,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get dueAt => $composableBuilder(
    column: $table.dueAt,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get userDisplayName => $composableBuilder(
    column: $table.userDisplayName,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get conversationTitle => $composableBuilder(
    column: $table.conversationTitle,
    builder: (column) => ColumnFilters(column),
  );
}

class $$RemindersTableOrderingComposer
    extends Composer<_$NomiDatabase, $RemindersTable> {
  $$RemindersTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get content => $composableBuilder(
    column: $table.content,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get taskType => $composableBuilder(
    column: $table.taskType,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get frequency => $composableBuilder(
    column: $table.frequency,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get status => $composableBuilder(
    column: $table.status,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get dueAt => $composableBuilder(
    column: $table.dueAt,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get userDisplayName => $composableBuilder(
    column: $table.userDisplayName,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get conversationTitle => $composableBuilder(
    column: $table.conversationTitle,
    builder: (column) => ColumnOrderings(column),
  );
}

class $$RemindersTableAnnotationComposer
    extends Composer<_$NomiDatabase, $RemindersTable> {
  $$RemindersTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<String> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get content =>
      $composableBuilder(column: $table.content, builder: (column) => column);

  GeneratedColumn<String> get taskType =>
      $composableBuilder(column: $table.taskType, builder: (column) => column);

  GeneratedColumn<String> get frequency =>
      $composableBuilder(column: $table.frequency, builder: (column) => column);

  GeneratedColumn<String> get status =>
      $composableBuilder(column: $table.status, builder: (column) => column);

  GeneratedColumn<DateTime> get dueAt =>
      $composableBuilder(column: $table.dueAt, builder: (column) => column);

  GeneratedColumn<DateTime> get createdAt =>
      $composableBuilder(column: $table.createdAt, builder: (column) => column);

  GeneratedColumn<String> get userDisplayName => $composableBuilder(
    column: $table.userDisplayName,
    builder: (column) => column,
  );

  GeneratedColumn<String> get conversationTitle => $composableBuilder(
    column: $table.conversationTitle,
    builder: (column) => column,
  );
}

class $$RemindersTableTableManager
    extends
        RootTableManager<
          _$NomiDatabase,
          $RemindersTable,
          Reminder,
          $$RemindersTableFilterComposer,
          $$RemindersTableOrderingComposer,
          $$RemindersTableAnnotationComposer,
          $$RemindersTableCreateCompanionBuilder,
          $$RemindersTableUpdateCompanionBuilder,
          (Reminder, BaseReferences<_$NomiDatabase, $RemindersTable, Reminder>),
          Reminder,
          PrefetchHooks Function()
        > {
  $$RemindersTableTableManager(_$NomiDatabase db, $RemindersTable table)
    : super(
        TableManagerState(
          db: db,
          table: table,
          createFilteringComposer:
              () => $$RemindersTableFilterComposer($db: db, $table: table),
          createOrderingComposer:
              () => $$RemindersTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer:
              () => $$RemindersTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback:
              ({
                Value<String> id = const Value.absent(),
                Value<String> content = const Value.absent(),
                Value<String?> taskType = const Value.absent(),
                Value<String?> frequency = const Value.absent(),
                Value<String> status = const Value.absent(),
                Value<DateTime> dueAt = const Value.absent(),
                Value<DateTime> createdAt = const Value.absent(),
                Value<String?> userDisplayName = const Value.absent(),
                Value<String?> conversationTitle = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => RemindersCompanion(
                id: id,
                content: content,
                taskType: taskType,
                frequency: frequency,
                status: status,
                dueAt: dueAt,
                createdAt: createdAt,
                userDisplayName: userDisplayName,
                conversationTitle: conversationTitle,
                rowid: rowid,
              ),
          createCompanionCallback:
              ({
                required String id,
                required String content,
                Value<String?> taskType = const Value.absent(),
                Value<String?> frequency = const Value.absent(),
                required String status,
                required DateTime dueAt,
                required DateTime createdAt,
                Value<String?> userDisplayName = const Value.absent(),
                Value<String?> conversationTitle = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => RemindersCompanion.insert(
                id: id,
                content: content,
                taskType: taskType,
                frequency: frequency,
                status: status,
                dueAt: dueAt,
                createdAt: createdAt,
                userDisplayName: userDisplayName,
                conversationTitle: conversationTitle,
                rowid: rowid,
              ),
          withReferenceMapper:
              (p0) =>
                  p0
                      .map(
                        (e) => (
                          e.readTable(table),
                          BaseReferences(db, table, e),
                        ),
                      )
                      .toList(),
          prefetchHooksCallback: null,
        ),
      );
}

typedef $$RemindersTableProcessedTableManager =
    ProcessedTableManager<
      _$NomiDatabase,
      $RemindersTable,
      Reminder,
      $$RemindersTableFilterComposer,
      $$RemindersTableOrderingComposer,
      $$RemindersTableAnnotationComposer,
      $$RemindersTableCreateCompanionBuilder,
      $$RemindersTableUpdateCompanionBuilder,
      (Reminder, BaseReferences<_$NomiDatabase, $RemindersTable, Reminder>),
      Reminder,
      PrefetchHooks Function()
    >;
typedef $$TransactionsTableCreateCompanionBuilder =
    TransactionsCompanion Function({
      required String id,
      Value<String?> merchantName,
      Value<String?> category,
      Value<String?> description,
      required String totalAmount,
      required DateTime createdAt,
      Value<String?> userDisplayName,
      Value<String?> conversationTitle,
      Value<String?> itemsJson,
      Value<int> rowid,
    });
typedef $$TransactionsTableUpdateCompanionBuilder =
    TransactionsCompanion Function({
      Value<String> id,
      Value<String?> merchantName,
      Value<String?> category,
      Value<String?> description,
      Value<String> totalAmount,
      Value<DateTime> createdAt,
      Value<String?> userDisplayName,
      Value<String?> conversationTitle,
      Value<String?> itemsJson,
      Value<int> rowid,
    });

class $$TransactionsTableFilterComposer
    extends Composer<_$NomiDatabase, $TransactionsTable> {
  $$TransactionsTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get merchantName => $composableBuilder(
    column: $table.merchantName,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get category => $composableBuilder(
    column: $table.category,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get description => $composableBuilder(
    column: $table.description,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get totalAmount => $composableBuilder(
    column: $table.totalAmount,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get userDisplayName => $composableBuilder(
    column: $table.userDisplayName,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get conversationTitle => $composableBuilder(
    column: $table.conversationTitle,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get itemsJson => $composableBuilder(
    column: $table.itemsJson,
    builder: (column) => ColumnFilters(column),
  );
}

class $$TransactionsTableOrderingComposer
    extends Composer<_$NomiDatabase, $TransactionsTable> {
  $$TransactionsTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get merchantName => $composableBuilder(
    column: $table.merchantName,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get category => $composableBuilder(
    column: $table.category,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get description => $composableBuilder(
    column: $table.description,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get totalAmount => $composableBuilder(
    column: $table.totalAmount,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get userDisplayName => $composableBuilder(
    column: $table.userDisplayName,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get conversationTitle => $composableBuilder(
    column: $table.conversationTitle,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get itemsJson => $composableBuilder(
    column: $table.itemsJson,
    builder: (column) => ColumnOrderings(column),
  );
}

class $$TransactionsTableAnnotationComposer
    extends Composer<_$NomiDatabase, $TransactionsTable> {
  $$TransactionsTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<String> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get merchantName => $composableBuilder(
    column: $table.merchantName,
    builder: (column) => column,
  );

  GeneratedColumn<String> get category =>
      $composableBuilder(column: $table.category, builder: (column) => column);

  GeneratedColumn<String> get description => $composableBuilder(
    column: $table.description,
    builder: (column) => column,
  );

  GeneratedColumn<String> get totalAmount => $composableBuilder(
    column: $table.totalAmount,
    builder: (column) => column,
  );

  GeneratedColumn<DateTime> get createdAt =>
      $composableBuilder(column: $table.createdAt, builder: (column) => column);

  GeneratedColumn<String> get userDisplayName => $composableBuilder(
    column: $table.userDisplayName,
    builder: (column) => column,
  );

  GeneratedColumn<String> get conversationTitle => $composableBuilder(
    column: $table.conversationTitle,
    builder: (column) => column,
  );

  GeneratedColumn<String> get itemsJson =>
      $composableBuilder(column: $table.itemsJson, builder: (column) => column);
}

class $$TransactionsTableTableManager
    extends
        RootTableManager<
          _$NomiDatabase,
          $TransactionsTable,
          Transaction,
          $$TransactionsTableFilterComposer,
          $$TransactionsTableOrderingComposer,
          $$TransactionsTableAnnotationComposer,
          $$TransactionsTableCreateCompanionBuilder,
          $$TransactionsTableUpdateCompanionBuilder,
          (
            Transaction,
            BaseReferences<_$NomiDatabase, $TransactionsTable, Transaction>,
          ),
          Transaction,
          PrefetchHooks Function()
        > {
  $$TransactionsTableTableManager(_$NomiDatabase db, $TransactionsTable table)
    : super(
        TableManagerState(
          db: db,
          table: table,
          createFilteringComposer:
              () => $$TransactionsTableFilterComposer($db: db, $table: table),
          createOrderingComposer:
              () => $$TransactionsTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer:
              () =>
                  $$TransactionsTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback:
              ({
                Value<String> id = const Value.absent(),
                Value<String?> merchantName = const Value.absent(),
                Value<String?> category = const Value.absent(),
                Value<String?> description = const Value.absent(),
                Value<String> totalAmount = const Value.absent(),
                Value<DateTime> createdAt = const Value.absent(),
                Value<String?> userDisplayName = const Value.absent(),
                Value<String?> conversationTitle = const Value.absent(),
                Value<String?> itemsJson = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => TransactionsCompanion(
                id: id,
                merchantName: merchantName,
                category: category,
                description: description,
                totalAmount: totalAmount,
                createdAt: createdAt,
                userDisplayName: userDisplayName,
                conversationTitle: conversationTitle,
                itemsJson: itemsJson,
                rowid: rowid,
              ),
          createCompanionCallback:
              ({
                required String id,
                Value<String?> merchantName = const Value.absent(),
                Value<String?> category = const Value.absent(),
                Value<String?> description = const Value.absent(),
                required String totalAmount,
                required DateTime createdAt,
                Value<String?> userDisplayName = const Value.absent(),
                Value<String?> conversationTitle = const Value.absent(),
                Value<String?> itemsJson = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => TransactionsCompanion.insert(
                id: id,
                merchantName: merchantName,
                category: category,
                description: description,
                totalAmount: totalAmount,
                createdAt: createdAt,
                userDisplayName: userDisplayName,
                conversationTitle: conversationTitle,
                itemsJson: itemsJson,
                rowid: rowid,
              ),
          withReferenceMapper:
              (p0) =>
                  p0
                      .map(
                        (e) => (
                          e.readTable(table),
                          BaseReferences(db, table, e),
                        ),
                      )
                      .toList(),
          prefetchHooksCallback: null,
        ),
      );
}

typedef $$TransactionsTableProcessedTableManager =
    ProcessedTableManager<
      _$NomiDatabase,
      $TransactionsTable,
      Transaction,
      $$TransactionsTableFilterComposer,
      $$TransactionsTableOrderingComposer,
      $$TransactionsTableAnnotationComposer,
      $$TransactionsTableCreateCompanionBuilder,
      $$TransactionsTableUpdateCompanionBuilder,
      (
        Transaction,
        BaseReferences<_$NomiDatabase, $TransactionsTable, Transaction>,
      ),
      Transaction,
      PrefetchHooks Function()
    >;
typedef $$HealthMetricsTableCreateCompanionBuilder =
    HealthMetricsCompanion Function({
      required String id,
      required String userId,
      required DateTime logDate,
      Value<int> steps,
      Value<int?> avgHeartRate,
      Value<double?> sleepHours,
      required DateTime updatedAt,
      Value<int> rowid,
    });
typedef $$HealthMetricsTableUpdateCompanionBuilder =
    HealthMetricsCompanion Function({
      Value<String> id,
      Value<String> userId,
      Value<DateTime> logDate,
      Value<int> steps,
      Value<int?> avgHeartRate,
      Value<double?> sleepHours,
      Value<DateTime> updatedAt,
      Value<int> rowid,
    });

class $$HealthMetricsTableFilterComposer
    extends Composer<_$NomiDatabase, $HealthMetricsTable> {
  $$HealthMetricsTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get userId => $composableBuilder(
    column: $table.userId,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get logDate => $composableBuilder(
    column: $table.logDate,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<int> get steps => $composableBuilder(
    column: $table.steps,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<int> get avgHeartRate => $composableBuilder(
    column: $table.avgHeartRate,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<double> get sleepHours => $composableBuilder(
    column: $table.sleepHours,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get updatedAt => $composableBuilder(
    column: $table.updatedAt,
    builder: (column) => ColumnFilters(column),
  );
}

class $$HealthMetricsTableOrderingComposer
    extends Composer<_$NomiDatabase, $HealthMetricsTable> {
  $$HealthMetricsTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get userId => $composableBuilder(
    column: $table.userId,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get logDate => $composableBuilder(
    column: $table.logDate,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get steps => $composableBuilder(
    column: $table.steps,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<int> get avgHeartRate => $composableBuilder(
    column: $table.avgHeartRate,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<double> get sleepHours => $composableBuilder(
    column: $table.sleepHours,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get updatedAt => $composableBuilder(
    column: $table.updatedAt,
    builder: (column) => ColumnOrderings(column),
  );
}

class $$HealthMetricsTableAnnotationComposer
    extends Composer<_$NomiDatabase, $HealthMetricsTable> {
  $$HealthMetricsTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<String> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get userId =>
      $composableBuilder(column: $table.userId, builder: (column) => column);

  GeneratedColumn<DateTime> get logDate =>
      $composableBuilder(column: $table.logDate, builder: (column) => column);

  GeneratedColumn<int> get steps =>
      $composableBuilder(column: $table.steps, builder: (column) => column);

  GeneratedColumn<int> get avgHeartRate => $composableBuilder(
    column: $table.avgHeartRate,
    builder: (column) => column,
  );

  GeneratedColumn<double> get sleepHours => $composableBuilder(
    column: $table.sleepHours,
    builder: (column) => column,
  );

  GeneratedColumn<DateTime> get updatedAt =>
      $composableBuilder(column: $table.updatedAt, builder: (column) => column);
}

class $$HealthMetricsTableTableManager
    extends
        RootTableManager<
          _$NomiDatabase,
          $HealthMetricsTable,
          HealthMetric,
          $$HealthMetricsTableFilterComposer,
          $$HealthMetricsTableOrderingComposer,
          $$HealthMetricsTableAnnotationComposer,
          $$HealthMetricsTableCreateCompanionBuilder,
          $$HealthMetricsTableUpdateCompanionBuilder,
          (
            HealthMetric,
            BaseReferences<_$NomiDatabase, $HealthMetricsTable, HealthMetric>,
          ),
          HealthMetric,
          PrefetchHooks Function()
        > {
  $$HealthMetricsTableTableManager(_$NomiDatabase db, $HealthMetricsTable table)
    : super(
        TableManagerState(
          db: db,
          table: table,
          createFilteringComposer:
              () => $$HealthMetricsTableFilterComposer($db: db, $table: table),
          createOrderingComposer:
              () =>
                  $$HealthMetricsTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer:
              () => $$HealthMetricsTableAnnotationComposer(
                $db: db,
                $table: table,
              ),
          updateCompanionCallback:
              ({
                Value<String> id = const Value.absent(),
                Value<String> userId = const Value.absent(),
                Value<DateTime> logDate = const Value.absent(),
                Value<int> steps = const Value.absent(),
                Value<int?> avgHeartRate = const Value.absent(),
                Value<double?> sleepHours = const Value.absent(),
                Value<DateTime> updatedAt = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => HealthMetricsCompanion(
                id: id,
                userId: userId,
                logDate: logDate,
                steps: steps,
                avgHeartRate: avgHeartRate,
                sleepHours: sleepHours,
                updatedAt: updatedAt,
                rowid: rowid,
              ),
          createCompanionCallback:
              ({
                required String id,
                required String userId,
                required DateTime logDate,
                Value<int> steps = const Value.absent(),
                Value<int?> avgHeartRate = const Value.absent(),
                Value<double?> sleepHours = const Value.absent(),
                required DateTime updatedAt,
                Value<int> rowid = const Value.absent(),
              }) => HealthMetricsCompanion.insert(
                id: id,
                userId: userId,
                logDate: logDate,
                steps: steps,
                avgHeartRate: avgHeartRate,
                sleepHours: sleepHours,
                updatedAt: updatedAt,
                rowid: rowid,
              ),
          withReferenceMapper:
              (p0) =>
                  p0
                      .map(
                        (e) => (
                          e.readTable(table),
                          BaseReferences(db, table, e),
                        ),
                      )
                      .toList(),
          prefetchHooksCallback: null,
        ),
      );
}

typedef $$HealthMetricsTableProcessedTableManager =
    ProcessedTableManager<
      _$NomiDatabase,
      $HealthMetricsTable,
      HealthMetric,
      $$HealthMetricsTableFilterComposer,
      $$HealthMetricsTableOrderingComposer,
      $$HealthMetricsTableAnnotationComposer,
      $$HealthMetricsTableCreateCompanionBuilder,
      $$HealthMetricsTableUpdateCompanionBuilder,
      (
        HealthMetric,
        BaseReferences<_$NomiDatabase, $HealthMetricsTable, HealthMetric>,
      ),
      HealthMetric,
      PrefetchHooks Function()
    >;
typedef $$PluginsTableCreateCompanionBuilder =
    PluginsCompanion Function({
      required String id,
      required String name,
      required String slug,
      Value<String?> description,
      Value<String?> scriptCode,
      Value<String?> version,
      Value<String?> author,
      Value<String?> intentsJson,
      required DateTime createdAt,
      required DateTime updatedAt,
      Value<int> rowid,
    });
typedef $$PluginsTableUpdateCompanionBuilder =
    PluginsCompanion Function({
      Value<String> id,
      Value<String> name,
      Value<String> slug,
      Value<String?> description,
      Value<String?> scriptCode,
      Value<String?> version,
      Value<String?> author,
      Value<String?> intentsJson,
      Value<DateTime> createdAt,
      Value<DateTime> updatedAt,
      Value<int> rowid,
    });

class $$PluginsTableFilterComposer
    extends Composer<_$NomiDatabase, $PluginsTable> {
  $$PluginsTableFilterComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnFilters<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get name => $composableBuilder(
    column: $table.name,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get slug => $composableBuilder(
    column: $table.slug,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get description => $composableBuilder(
    column: $table.description,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get scriptCode => $composableBuilder(
    column: $table.scriptCode,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get version => $composableBuilder(
    column: $table.version,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get author => $composableBuilder(
    column: $table.author,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<String> get intentsJson => $composableBuilder(
    column: $table.intentsJson,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnFilters(column),
  );

  ColumnFilters<DateTime> get updatedAt => $composableBuilder(
    column: $table.updatedAt,
    builder: (column) => ColumnFilters(column),
  );
}

class $$PluginsTableOrderingComposer
    extends Composer<_$NomiDatabase, $PluginsTable> {
  $$PluginsTableOrderingComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  ColumnOrderings<String> get id => $composableBuilder(
    column: $table.id,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get name => $composableBuilder(
    column: $table.name,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get slug => $composableBuilder(
    column: $table.slug,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get description => $composableBuilder(
    column: $table.description,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get scriptCode => $composableBuilder(
    column: $table.scriptCode,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get version => $composableBuilder(
    column: $table.version,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get author => $composableBuilder(
    column: $table.author,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<String> get intentsJson => $composableBuilder(
    column: $table.intentsJson,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get createdAt => $composableBuilder(
    column: $table.createdAt,
    builder: (column) => ColumnOrderings(column),
  );

  ColumnOrderings<DateTime> get updatedAt => $composableBuilder(
    column: $table.updatedAt,
    builder: (column) => ColumnOrderings(column),
  );
}

class $$PluginsTableAnnotationComposer
    extends Composer<_$NomiDatabase, $PluginsTable> {
  $$PluginsTableAnnotationComposer({
    required super.$db,
    required super.$table,
    super.joinBuilder,
    super.$addJoinBuilderToRootComposer,
    super.$removeJoinBuilderFromRootComposer,
  });
  GeneratedColumn<String> get id =>
      $composableBuilder(column: $table.id, builder: (column) => column);

  GeneratedColumn<String> get name =>
      $composableBuilder(column: $table.name, builder: (column) => column);

  GeneratedColumn<String> get slug =>
      $composableBuilder(column: $table.slug, builder: (column) => column);

  GeneratedColumn<String> get description => $composableBuilder(
    column: $table.description,
    builder: (column) => column,
  );

  GeneratedColumn<String> get scriptCode => $composableBuilder(
    column: $table.scriptCode,
    builder: (column) => column,
  );

  GeneratedColumn<String> get version =>
      $composableBuilder(column: $table.version, builder: (column) => column);

  GeneratedColumn<String> get author =>
      $composableBuilder(column: $table.author, builder: (column) => column);

  GeneratedColumn<String> get intentsJson => $composableBuilder(
    column: $table.intentsJson,
    builder: (column) => column,
  );

  GeneratedColumn<DateTime> get createdAt =>
      $composableBuilder(column: $table.createdAt, builder: (column) => column);

  GeneratedColumn<DateTime> get updatedAt =>
      $composableBuilder(column: $table.updatedAt, builder: (column) => column);
}

class $$PluginsTableTableManager
    extends
        RootTableManager<
          _$NomiDatabase,
          $PluginsTable,
          Plugin,
          $$PluginsTableFilterComposer,
          $$PluginsTableOrderingComposer,
          $$PluginsTableAnnotationComposer,
          $$PluginsTableCreateCompanionBuilder,
          $$PluginsTableUpdateCompanionBuilder,
          (Plugin, BaseReferences<_$NomiDatabase, $PluginsTable, Plugin>),
          Plugin,
          PrefetchHooks Function()
        > {
  $$PluginsTableTableManager(_$NomiDatabase db, $PluginsTable table)
    : super(
        TableManagerState(
          db: db,
          table: table,
          createFilteringComposer:
              () => $$PluginsTableFilterComposer($db: db, $table: table),
          createOrderingComposer:
              () => $$PluginsTableOrderingComposer($db: db, $table: table),
          createComputedFieldComposer:
              () => $$PluginsTableAnnotationComposer($db: db, $table: table),
          updateCompanionCallback:
              ({
                Value<String> id = const Value.absent(),
                Value<String> name = const Value.absent(),
                Value<String> slug = const Value.absent(),
                Value<String?> description = const Value.absent(),
                Value<String?> scriptCode = const Value.absent(),
                Value<String?> version = const Value.absent(),
                Value<String?> author = const Value.absent(),
                Value<String?> intentsJson = const Value.absent(),
                Value<DateTime> createdAt = const Value.absent(),
                Value<DateTime> updatedAt = const Value.absent(),
                Value<int> rowid = const Value.absent(),
              }) => PluginsCompanion(
                id: id,
                name: name,
                slug: slug,
                description: description,
                scriptCode: scriptCode,
                version: version,
                author: author,
                intentsJson: intentsJson,
                createdAt: createdAt,
                updatedAt: updatedAt,
                rowid: rowid,
              ),
          createCompanionCallback:
              ({
                required String id,
                required String name,
                required String slug,
                Value<String?> description = const Value.absent(),
                Value<String?> scriptCode = const Value.absent(),
                Value<String?> version = const Value.absent(),
                Value<String?> author = const Value.absent(),
                Value<String?> intentsJson = const Value.absent(),
                required DateTime createdAt,
                required DateTime updatedAt,
                Value<int> rowid = const Value.absent(),
              }) => PluginsCompanion.insert(
                id: id,
                name: name,
                slug: slug,
                description: description,
                scriptCode: scriptCode,
                version: version,
                author: author,
                intentsJson: intentsJson,
                createdAt: createdAt,
                updatedAt: updatedAt,
                rowid: rowid,
              ),
          withReferenceMapper:
              (p0) =>
                  p0
                      .map(
                        (e) => (
                          e.readTable(table),
                          BaseReferences(db, table, e),
                        ),
                      )
                      .toList(),
          prefetchHooksCallback: null,
        ),
      );
}

typedef $$PluginsTableProcessedTableManager =
    ProcessedTableManager<
      _$NomiDatabase,
      $PluginsTable,
      Plugin,
      $$PluginsTableFilterComposer,
      $$PluginsTableOrderingComposer,
      $$PluginsTableAnnotationComposer,
      $$PluginsTableCreateCompanionBuilder,
      $$PluginsTableUpdateCompanionBuilder,
      (Plugin, BaseReferences<_$NomiDatabase, $PluginsTable, Plugin>),
      Plugin,
      PrefetchHooks Function()
    >;

class $NomiDatabaseManager {
  final _$NomiDatabase _db;
  $NomiDatabaseManager(this._db);
  $$ConversationsTableTableManager get conversations =>
      $$ConversationsTableTableManager(_db, _db.conversations);
  $$MessagesTableTableManager get messages =>
      $$MessagesTableTableManager(_db, _db.messages);
  $$RemindersTableTableManager get reminders =>
      $$RemindersTableTableManager(_db, _db.reminders);
  $$TransactionsTableTableManager get transactions =>
      $$TransactionsTableTableManager(_db, _db.transactions);
  $$HealthMetricsTableTableManager get healthMetrics =>
      $$HealthMetricsTableTableManager(_db, _db.healthMetrics);
  $$PluginsTableTableManager get plugins =>
      $$PluginsTableTableManager(_db, _db.plugins);
}
