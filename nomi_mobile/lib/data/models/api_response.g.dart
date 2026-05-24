// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'api_response.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ApiResponse<T> _$ApiResponseFromJson<T>(
  Map<String, dynamic> json,
  T Function(Object? json) fromJsonT,
) => ApiResponse<T>(
  meta: Meta.fromJson(json['meta'] as Map<String, dynamic>),
  data: _$nullableGenericFromJson(json['data'], fromJsonT),
);

Map<String, dynamic> _$ApiResponseToJson<T>(
  ApiResponse<T> instance,
  Object? Function(T value) toJsonT,
) => <String, dynamic>{
  'meta': instance.meta,
  'data': _$nullableGenericToJson(instance.data, toJsonT),
};

T? _$nullableGenericFromJson<T>(
  Object? input,
  T Function(Object? json) fromJson,
) => input == null ? null : fromJson(input);

Object? _$nullableGenericToJson<T>(
  T? input,
  Object? Function(T value) toJson,
) => input == null ? null : toJson(input);

Meta _$MetaFromJson(Map<String, dynamic> json) => Meta(
  code: (json['code'] as num).toInt(),
  status: json['status'] as String?,
  message: json['message'] as String,
);

Map<String, dynamic> _$MetaToJson(Meta instance) => <String, dynamic>{
  'code': instance.code,
  'status': instance.status,
  'message': instance.message,
};
