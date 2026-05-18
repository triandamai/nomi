package id.nomi.trianapp.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class ConversationDto(
    val id: String,
    val name: String,
    @SerialName("cumulative_tokens") val cumulativeTokens: Long,
    @SerialName("max_token_usage") val maxTokenUsage: Long = 0,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String
)

@Serializable
data class UpdateConversationRequest(
    @SerialName("max_token_usage") val maxTokenUsage: Long
)