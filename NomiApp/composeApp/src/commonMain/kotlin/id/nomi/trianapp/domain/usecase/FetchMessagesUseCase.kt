package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.local.MessageEntity
import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.MessageListWrapper
import id.nomi.trianapp.data.remote.ApiClient
import kotlinx.coroutines.flow.Flow

class FetchMessagesUseCase(
    private val apiClient: ApiClient,
    private val nomiDb: NomiDb
) {
    suspend operator fun invoke(
        conversationId: String,
        limit: Int = 20
    ): ApiResponse<MessageListWrapper> {
        val response = apiClient.get<MessageListWrapper>("/api/conversations/$conversationId/messages?limit=$limit")

        val messages = response.data?.messages?.map { dto->
            MessageEntity(
                id = dto.id,
                conversationId = dto.conversationId,
                displayName = dto.displayName ?: "",
                role = dto.role,
                content = dto.content,
                totalTokens = dto.totalTokens ?: 0,
                answerTokens = dto.answerTokens ?: 0,
                promptTokens = dto.promptTokens ?: 0,
                thought = dto.thought,
                imageUrl = dto.imageUrl,
                videoUrl = dto.videoUrl,
                audioUrl = dto.audioUrl,
                documentUrl = dto.documentUrl,
                stickerUrl = dto.stickerUrl,
                userId = dto.userId,
                createdAt = dto.createdAt
            )
        } ?: listOf()
        nomiDb.nomiDao().insertMessages(messages)
        return response
    }

    fun getLocalMessages(conversationId: String): Flow<List<MessageEntity>> {
        return nomiDb.nomiDao().getMessages(conversationId)
    }
}
