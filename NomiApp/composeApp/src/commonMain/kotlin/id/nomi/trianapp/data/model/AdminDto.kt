package id.nomi.trianapp.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class AdminUserDto(
    val id: String,
    val name: String? = null,
    @SerialName("display_name") val displayName: String,
    val email: String? = null,
    val role: String,
    @SerialName("is_verified") val isVerified: Boolean,
    @SerialName("created_at") val createdAt: String
)

@Serializable
data class AdminUserListResponse(
    val items: List<AdminUserDto>,
    @SerialName("next_cursor") val nextCursor: String? = null
)

@Serializable
data class AdminChannelDto(
    val id: String,
    @SerialName("channel_type") val channelType: String,
    @SerialName("external_id") val externalId: String,
    @SerialName("external_chat_id") val externalChatId: String,
    @SerialName("conversation_title") val conversationTitle: String? = null
)

@Serializable
data class AdminConversationDto(
    @SerialName("conversation_id") val conversationId: String,
    val title: String,
    @SerialName("joined_at") val joinedAt: String
)

@Serializable
data class AdminUserDetailResponse(
    val user: AdminUserDto,
    val channels: List<AdminChannelDto>,
    val conversations: List<AdminConversationDto>
)
