package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.model.AdminUserDetailResponse
import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.remote.ApiClient

class GetAdminUserDetailUseCase(
    private val apiClient: ApiClient
) {
    suspend operator fun invoke(userId: String): ApiResponse<AdminUserDetailResponse> {
        return apiClient.get("/api/v1/admin/users/$userId")
    }
}
