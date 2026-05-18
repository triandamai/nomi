package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.local.ConversationEntity
import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.ConversationDto
import id.nomi.trianapp.data.remote.ApiClient
import kotlinx.coroutines.flow.Flow

class GetConversationsUseCase(
    private val apiClient: ApiClient,
    private val nomiDb: NomiDb
) {
    suspend operator fun invoke(): ApiResponse<List<ConversationDto>> {
        val response = apiClient.get<List<ConversationDto>>("/api/conversations")

        response.data?.let { conversations ->
            val entities = conversations.map {
                ConversationEntity(
                    id = it.id,
                    name = it.name,
                    cumulativeTokens = it.cumulativeTokens,
                    createdAt = it.createdAt,
                    updatedAt = it.updatedAt
                )
            }
            nomiDb.nomiDao().insertConversations(entities)
        }

        return response
    }

    fun getLocalConversations(): Flow<List<ConversationEntity>> {
        return nomiDb.nomiDao().getConversations()
    }
}
