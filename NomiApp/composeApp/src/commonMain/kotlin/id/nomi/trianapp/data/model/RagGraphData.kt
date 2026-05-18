package id.nomi.trianapp.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class RagGraphData(
    val nodes: List<RagNodeDto>,
    val links: List<RagLinkDto>
)

@Serializable
data class RagNodeDto(
    val id: String,
    val label: String,
    @SerialName("node_type") val nodeType: String,
    val color: String,
    @SerialName("conversation_id") val conversationId: String? = null
)

@Serializable
data class RagLinkDto(
    val source: String,
    val target: String,
    val relationship: String
)
