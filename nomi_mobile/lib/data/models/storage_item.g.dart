// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'storage_item.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

StorageItem _$StorageItemFromJson(Map<String, dynamic> json) => StorageItem(
  path: json['path'] as String?,
  full_path: json['full_path'] as String?,
  type: json['type'] as String?,
  name: json['name'] as String?,
  size: (json['size'] as num?)?.toInt(),
  mime_type: json['mime_type'] as String?,
);

Map<String, dynamic> _$StorageItemToJson(StorageItem instance) =>
    <String, dynamic>{
      'path': instance.path,
      'full_path': instance.full_path,
      'type': instance.type,
      'name': instance.name,
      'size': instance.size,
      'mime_type': instance.mime_type,
    };
