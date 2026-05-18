package id.nomi.trianapp.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class TokenUpdateDto (
    @SerialName("conversation_id")
    val conversationID: String,

    @SerialName("cumulative_tokens")
    val cumulativeTokens: Long
)
