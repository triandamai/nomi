package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.model.SendMessageRequest
import id.nomi.trianapp.data.remote.ApiClient

class SendMessageUseCase(
    private val apiClient: ApiClient
) {
    suspend operator fun invoke(conversationId: String, content: String): Result<Unit> {
        return try {
            val response = apiClient.post<SendMessageRequest, Unit>(
                "/api/chat/stream",
                SendMessageRequest(conversationId,content)
            )
            if (response.meta.code in 200..209) {
                Result.success(Unit)
            } else {
                Result.failure(Exception(response.meta.message))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
