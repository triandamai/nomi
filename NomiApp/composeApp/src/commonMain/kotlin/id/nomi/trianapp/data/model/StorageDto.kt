package id.nomi.trianapp.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class StorageItemDto(
    val type: String, // "folder" or "file"
    val name: String,
    @SerialName("full_path") val fullPath: String,
    val size: Long? = null,
    @SerialName("updated_at") val updatedAt: String? = null,
    @SerialName("content_type") val contentType: String? = null
)
