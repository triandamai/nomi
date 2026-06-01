import 'package:nomi_mobile/data/models/api_response.dart';
import 'package:nomi_mobile/core/api/api_client.dart';
import '../models/friend.dart';

class FriendRepository {
  final ApiClient _apiClient;

  FriendRepository(this._apiClient);

  Future<ApiResponse<List<FriendProfile>>> getFriends() async {
    final response = await _apiClient.dio.get('/friends');
    return ApiResponse.fromJson(
      response.data,
      (json) => (json as List)
          .map((item) => FriendProfile.fromJson(item as Map<String, dynamic>))
          .toList(),
    );
  }

  Future<ApiResponse<void>> sendFriendRequest(String receiverId) async {
    final response = await _apiClient.dio.post(
      '/friends/requests',
      data: {'receiver_id': receiverId},
    );
    return ApiResponse.fromJson(
      response.data,
      (json) => null,
    );
  }

  Future<ApiResponse<String?>> respondFriendRequest(
    String senderId,
    bool accept,
  ) async {
    final response = await _apiClient.dio.post(
      '/friends/requests/respond',
      data: {'sender_id': senderId, 'accept': accept},
    );
    return ApiResponse.fromJson(
      response.data,
      (json) => json as String?,
    );
  }

  Future<ApiResponse<Map<String, List<FriendRequestItem>>>> getPendingRequests() async {
    final response = await _apiClient.dio.get('/friends/requests/pending');
    return ApiResponse.fromJson(
      response.data,
      (json) {
        final data = json as Map<String, dynamic>;
        final incoming = (data['incoming'] as List)
            .map((item) => FriendRequestItem.fromJson(item as Map<String, dynamic>))
            .toList();
        final outgoing = (data['outgoing'] as List)
            .map((item) => FriendRequestItem.fromJson(item as Map<String, dynamic>))
            .toList();
        return {
          'incoming': incoming,
          'outgoing': outgoing,
        };
      },
    );
  }

  Future<ApiResponse<void>> blockUser(String blockedUserId) async {
    final response = await _apiClient.dio.post(
      '/friends/block',
      data: {'blocked_user_id': blockedUserId},
    );
    return ApiResponse.fromJson(
      response.data,
      (json) => null,
    );
  }
}
