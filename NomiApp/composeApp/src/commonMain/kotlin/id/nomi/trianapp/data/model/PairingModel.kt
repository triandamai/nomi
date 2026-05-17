package id.nomi.trianapp.data.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class PairingResponse(
    @SerialName("access_token") val accessToken: String,
    @SerialName("user_id") val userId: String,
    val profile: UserProfile,
    val channels: List<ChannelResponse>,
    val conversations: List<ConversationResponse>
)

@Serializable
data class UserProfile(
    val id: String,
    @SerialName("display_name") val displayName: String,
    @SerialName("avatar_url") val avatarUrl: String? = null,
    val role: String
)

@Serializable
data class ChannelResponse(
    val paired: Boolean,
    val platform: String
)

@Serializable
data class ConversationResponse(
    val id: String,
    val name: String,
    @SerialName("cumulative_tokens") val cumulativeTokens: Long,
    @SerialName("created_at") val createdAt: String,
    @SerialName("updated_at") val updatedAt: String
)

@Serializable
data class PairingRequest(
    @SerialName("pairing_code") val pairingCode: String
)
