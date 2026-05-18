package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.StorageItemDto
import id.nomi.trianapp.data.remote.ApiClient

class ExploreStorageUseCase(
    private val apiClient: ApiClient
) {
    suspend operator fun invoke(prefix: String? = null): ApiResponse<List<StorageItemDto>> {
        val url = if (prefix != null) "/api/v1/admin/storage/explore?prefix=$prefix" else "/api/v1/admin/storage/explore"
        return apiClient.get(url)
    }
}
