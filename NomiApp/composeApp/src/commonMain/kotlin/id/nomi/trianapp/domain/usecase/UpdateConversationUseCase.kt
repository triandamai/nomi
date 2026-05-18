package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.ConversationDto
import id.nomi.trianapp.data.model.UpdateConversationRequest
import id.nomi.trianapp.data.remote.ApiClient

class UpdateConversationUseCase(
    private val apiClient: ApiClient
) {
    suspend operator fun invoke(conversationId: String, maxTokenUsage: Long): ApiResponse<ConversationDto> {
        return apiClient.patch(
            "/api/conversations/$conversationId",
            UpdateConversationRequest(maxTokenUsage = maxTokenUsage)
        )
    }
}
