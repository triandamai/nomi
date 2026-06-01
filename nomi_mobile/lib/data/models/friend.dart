class FriendProfile {
  final String id;
  final String? name;
  final String? displayName;
  final String? email;

  FriendProfile({required this.id, this.name, this.displayName, this.email});

  factory FriendProfile.fromJson(Map<String, dynamic> json) {
    return FriendProfile(
      id: json['id'] as String,
      name: json['name'] as String?,
      displayName: json['display_name'] as String?,
      email: json['email'] as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'display_name': displayName,
      'email': email,
    };
  }
}

class FriendRequestItem {
  final String id;
  final String senderId;
  final String receiverId;
  final String? senderDisplayName;
  final String? receiverDisplayName;
  final DateTime createdAt;

  FriendRequestItem({
    required this.id,
    required this.senderId,
    required this.receiverId,
    this.senderDisplayName,
    this.receiverDisplayName,
    required this.createdAt,
  });

  factory FriendRequestItem.fromJson(Map<String, dynamic> json) {
    return FriendRequestItem(
      id: json['id'] as String,
      senderId: json['sender_id'] as String,
      receiverId: json['receiver_id'] as String,
      senderDisplayName: json['sender_display_name'] as String?,
      receiverDisplayName: json['receiver_display_name'] as String?,
      createdAt: DateTime.parse(json['created_at'] as String),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'sender_id': senderId,
      'receiver_id': receiverId,
      'sender_display_name': senderDisplayName,
      'receiver_display_name': receiverDisplayName,
      'created_at': createdAt.toIso8601String(),
    };
  }
}
