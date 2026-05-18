package id.nomi.trianapp.data.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class MessageListWrapper(val messages: List<MessageDto>)

@Serializable
data class MessageDto(
    @SerialName("audio_url")
    val audioUrl: String? = null,

    val content: String,

    @SerialName("conversation_id")
    val conversationId: String,

    @SerialName("created_at")
    val createdAt: String,

    @SerialName("display_name")
    val displayName: String?=null,

    @SerialName("document_url")
    val documentUrl: String? = null,

    val id: String,

    @SerialName("image_url")
    val imageUrl: String? = null,

    val role: String,

    @SerialName("sticker_url")
    val stickerUrl: String? = null,

    val thought: String? = null,

    @SerialName("total_tokens")
    val totalTokens: Long? = null,
    @SerialName("answer_tokens")
    val answerTokens: Long? = null,
    @SerialName("prompt_tokens")
    val promptTokens: Long? = null,

    @SerialName("user_id")
    val userId: String?=null,

    @SerialName("video_url")
    val videoUrl: String? = null
)
