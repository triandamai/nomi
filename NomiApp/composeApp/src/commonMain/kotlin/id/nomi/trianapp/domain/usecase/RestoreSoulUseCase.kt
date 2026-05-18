package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.remote.ApiClient

class RestoreSoulUseCase(
    private val apiClient: ApiClient
) {
    suspend operator fun invoke(conversationId: String, soulId: String): ApiResponse<Unit> {
        return apiClient.post("/api/conversations/$conversationId/soul-history/$soulId/restore", emptyMap<String, String>())
    }
}
