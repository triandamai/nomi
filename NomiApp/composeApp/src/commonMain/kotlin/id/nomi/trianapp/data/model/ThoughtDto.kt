package id.nomi.trianapp.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class ThoughtDto(
    @SerialName("conversation_id")
    val conversationId: String,
    val text: String
)
