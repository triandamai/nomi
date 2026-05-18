package id.nomi.trianapp.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class PairingDto(
    @SerialName("access_token") val accessToken: String,
    @SerialName("user_id") val userId: String,
    val profile: UserProfileDto,
    val channels: List<ChannelDto>,
    val conversations: List<ConversationDto>
)