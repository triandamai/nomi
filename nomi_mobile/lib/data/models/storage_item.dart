import 'package:json_annotation/json_annotation.dart';

part 'storage_item.g.dart';

@JsonSerializable()
class StorageItem {
  final String? path;
  final String? full_path;
  final String? type; // 'bucket' or 'file'
  final String? name;
  final int? size;
  final String? mime_type;

  StorageItem({
    this.path,
    this.full_path,
    this.type,
    this.name,
    this.size,
    this.mime_type,
  });

  bool get isDir => type == 'bucket' || type == 'dir';
  String get displayPath => full_path ?? path ?? name ?? 'Unknown';

  factory StorageItem.fromJson(Map<String, dynamic> json) => _$StorageItemFromJson(json);
  Map<String, dynamic> toJson() => _$StorageItemToJson(this);
}
