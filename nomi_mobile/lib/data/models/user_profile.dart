import 'package:json_annotation/json_annotation.dart';

part 'user_profile.g.dart';

@JsonSerializable()
class UserProfile {
  final String id;
  final String? name;
  @JsonKey(name: 'display_name')
  final String? displayName;
  final String? email;
  final String? role;
  @JsonKey(name: 'is_verified')
  final bool? isVerified;
  @JsonKey(name: 'created_at')
  final String? createdAt;

  UserProfile({
    required this.id,
    this.name,
    this.displayName,
    this.email,
    this.role,
    this.isVerified,
    this.createdAt,
  });

  factory UserProfile.fromJson(Map<String, dynamic> json) => _$UserProfileFromJson(json);
  Map<String, dynamic> toJson() => _$UserProfileToJson(this);
}
