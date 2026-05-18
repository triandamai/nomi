package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.local.ConversationEntity
import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.model.ConversationDto
import id.nomi.trianapp.data.remote.ApiClient
import kotlinx.coroutines.flow.Flow

class GetConversationUseCase(
    private val apiClient: ApiClient,
    private val nomiDb: NomiDb
) {
    suspend operator fun invoke(id: String): Result<Unit> {
        return try {
            val response = apiClient.get<ConversationDto>("/api/conversations/$id")
            if (response.meta.code in 200..209 && response.data != null) {
                val dto = response.data
                val entity = ConversationEntity(
                    id = dto.id,
                    name = dto.name,
                    cumulativeTokens = dto.cumulativeTokens,
                    createdAt = dto.createdAt,
                    updatedAt = dto.updatedAt
                )
                nomiDb.nomiDao().insertConversations(listOf(entity))
                Result.success(Unit)
            } else {
                Result.failure(Exception(response.meta.message ?: "Failed to fetch conversation"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun getLocalConversation(id: String): Flow<ConversationEntity?> {
        return nomiDb.nomiDao().getConversation(id)
    }
}
