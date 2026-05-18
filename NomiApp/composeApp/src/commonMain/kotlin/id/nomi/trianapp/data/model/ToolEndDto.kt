package id.nomi.trianapp.data.model

import kotlinx.serialization.*
import kotlinx.serialization.json.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*

@Serializable
data class ToolEndDto(
    @SerialName("conversation_id")
    val conversationId: String,
    @SerialName("name")
    val name: String,
    @SerialName("text")
    val text: String?=null,
    @SerialName("success")
    val success: Boolean? = null
)
