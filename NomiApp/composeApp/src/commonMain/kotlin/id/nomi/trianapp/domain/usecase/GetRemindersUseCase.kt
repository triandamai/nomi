package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.local.ReminderEntity
import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.ReminderDto
import id.nomi.trianapp.data.remote.ApiClient
import kotlinx.coroutines.flow.Flow

class GetRemindersUseCase(
    private val apiClient: ApiClient,
    private val nomiDb: NomiDb
) {
    suspend operator fun invoke(limit: Int = 20): ApiResponse<List<ReminderDto>> {
        val response = try {
            apiClient.get<List<ReminderDto>>("/api/reminders?limit=$limit")
        } catch (e: Exception) {
            ApiResponse(
                data = null,
                meta = id.nomi.trianapp.data.model.MetaResponse(500, e.message ?: "Network Error")
            )
        }

        if (response.data != null) {
            val entities = response.data.map { dto ->
                ReminderEntity(
                    id = dto.id,
                    taskType = dto.taskType,
                    content = dto.content,
                    dueAt = dto.dueAt,
                    frequency = dto.frequency,
                    status = dto.status,
                    userDisplayName = dto.userDisplayName,
                    conversationTitle = dto.conversationTitle,
                    createdAt = dto.createdAt
                )
            }
            nomiDb.nomiDao().deleteReminders()
            nomiDb.nomiDao().insertReminders(entities)
        }

        return ApiResponse(
            data = response.data ?: listOf(),
            meta = response.meta,
            errors = response.errors
        )
    }

    fun getLocalReminders(): Flow<List<ReminderEntity>> {
        return nomiDb.nomiDao().getReminders()
    }
}
