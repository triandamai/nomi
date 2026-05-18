package id.nomi.trianapp.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class SoulHistoryDto(
    val id: String,
    val version: Int,
    @SerialName("change_reason") val changeReason: String,
    @SerialName("soul_content") val soulContent: String,
    @SerialName("created_at") val createdAt: String
)
