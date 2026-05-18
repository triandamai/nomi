package id.nomi.trianapp.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class SendMessageRequest(
    @SerialName("conversation_id")
    val conversationId: String,
    @SerialName("message")
    val message: String
)
