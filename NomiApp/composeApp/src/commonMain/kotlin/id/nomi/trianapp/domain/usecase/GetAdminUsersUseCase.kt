package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.model.AdminUserListResponse
import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.remote.ApiClient

class GetAdminUsersUseCase(
    private val apiClient: ApiClient
) {
    suspend operator fun invoke(limit: Int = 20): ApiResponse<AdminUserListResponse> {
        return apiClient.get("/api/v1/admin/users?limit=$limit")
    }
}
