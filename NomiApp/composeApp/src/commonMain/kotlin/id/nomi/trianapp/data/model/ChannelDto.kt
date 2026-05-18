package id.nomi.trianapp.data.model

import kotlinx.serialization.Serializable

@Serializable
data class ChannelDto(
    val paired: Boolean,
    val platform: String
)