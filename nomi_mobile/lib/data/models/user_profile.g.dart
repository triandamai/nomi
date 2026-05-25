// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'user_profile.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

UserProfile _$UserProfileFromJson(Map<String, dynamic> json) => UserProfile(
  id: json['id'] as String,
  name: json['name'] as String?,
  displayName: json['display_name'] as String?,
  email: json['email'] as String?,
  role: json['role'] as String?,
  isVerified: json['is_verified'] as bool?,
  createdAt: json['created_at'] as String?,
);

Map<String, dynamic> _$UserProfileToJson(UserProfile instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'display_name': instance.displayName,
      'email': instance.email,
      'role': instance.role,
      'is_verified': instance.isVerified,
      'created_at': instance.createdAt,
    };
