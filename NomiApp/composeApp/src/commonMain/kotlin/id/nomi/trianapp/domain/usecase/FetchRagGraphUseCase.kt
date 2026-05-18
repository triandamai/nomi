package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.model.RagGraphData
import id.nomi.trianapp.data.remote.ApiClient
import io.ktor.client.plugins.*

class FetchRagGraphUseCase(
    private val apiClient: ApiClient
) {
    suspend operator fun invoke(conversationId: String): Result<RagGraphData> {
        return try {
            val response = apiClient.get<RagGraphData>("/api/graph?conversation_id=$conversationId")
            if (response.data != null) {
                Result.success(response.data)
            } else {
                Result.failure(Exception(response.meta.message))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
