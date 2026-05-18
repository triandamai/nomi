package id.nomi.trianapp.data.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class ReminderListWrapper(val reminders: List<ReminderDto>)

@Serializable
data class ReminderDto(
    val id: String,
    @SerialName("task_type")
    val taskType: String,
    val payload: ReminderPayloadDto,
    val content: String,
    @SerialName("due_at")
    val dueAt: String,
    val frequency: String,
    val status: String,
    @SerialName("user_display_name")
    val userDisplayName: String,
    @SerialName("conversation_title")
    val conversationTitle: String,
    @SerialName("created_at")
    val createdAt: String
)

@Serializable
data class ReminderPayloadDto(
    val message: String?=null
)
