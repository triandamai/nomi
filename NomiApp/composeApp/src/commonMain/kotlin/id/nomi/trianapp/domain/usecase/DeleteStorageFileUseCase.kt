package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.remote.ApiClient

class DeleteStorageFileUseCase(
    private val apiClient: ApiClient
) {
    suspend operator fun invoke(fullPath: String): ApiResponse<Unit> {
        return apiClient.delete("/api/v1/admin/storage/file?path=$fullPath")
    }
}
