import 'package:nomi_mobile/data/models/api_response.dart';
import 'package:nomi_mobile/data/models/conversation.dart';
import 'package:nomi_mobile/core/api/api_client.dart';

class AuthRepository {
  final ApiClient _apiClient;

  AuthRepository(this._apiClient);

  Future<ApiResponse<void>> requestOtp(String identity, String channel) async {
    final response = await _apiClient.dio.post(
      '/auth/request-otp',
      data: {'identity': identity, 'channel': channel},
    );
    return ApiResponse.fromJson(
      response.data,
      (json) => null,
    );
  }

  Future<ApiResponse<Map<String, dynamic>>> verifyOtp(
    String identity,
    String code,
  ) async {
    final response = await _apiClient.dio.post(
      '/auth/verify-otp',
      data: {'identity': identity, 'code': code},
    );
    return ApiResponse.fromJson(
      response.data,
      (json) => json as Map<String, dynamic>,
    );
  }

  Future<ApiResponse<Profile>> getProfile() async {
    final response = await _apiClient.dio.get('/auth/profile');
    return ApiResponse.fromJson(
      response.data,
      (json) => Profile.fromJson(json as Map<String, dynamic>),
    );
  }

  Future<ApiResponse<void>> updateProfile(String displayName) async {
    final response = await _apiClient.dio.put(
      '/auth/profile',
      data: {'display_name': displayName},
    );
    return ApiResponse.fromJson(
      response.data,
      (json) => null,
    );
  }
}
