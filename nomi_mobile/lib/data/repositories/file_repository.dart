import 'dart:io';
import 'package:dio/dio.dart';
import 'package:nomi_mobile/core/api/api_client.dart';
import 'package:nomi_mobile/data/models/api_response.dart';

class FileRepository {
  final ApiClient _apiClient;

  FileRepository(this._apiClient);

  Future<ApiResponse<String>> uploadFile(File file) async {
    final String fileName = file.path.split('/').last;
    final FormData formData = FormData.fromMap({
      "file": await MultipartFile.fromFile(file.path, filename: fileName),
    });

    final response = await _apiClient.dio.post(
      '/upload',
      data: formData,
    );

    return ApiResponse.fromJson(
      response.data,
      (json) => json.toString(),
    );
  }
}
