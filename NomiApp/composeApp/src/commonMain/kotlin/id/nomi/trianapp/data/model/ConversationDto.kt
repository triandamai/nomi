package id.nomi.trianapp.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class ConversationDto(
    val id: String,
    val name: String,
    @SerialName("cumulative_tokens") val cumulativeTokens: Long,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String
)