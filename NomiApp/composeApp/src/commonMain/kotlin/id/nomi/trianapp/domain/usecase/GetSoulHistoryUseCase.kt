package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.SoulHistoryDto
import id.nomi.trianapp.data.remote.ApiClient

class GetSoulHistoryUseCase(
    private val apiClient: ApiClient
) {
    suspend operator fun invoke(conversationId: String): ApiResponse<List<SoulHistoryDto>> {
        return apiClient.get("/api/conversations/$conversationId/soul-history")
    }
}
