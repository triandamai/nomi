package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.local.ProfileEntity
import id.nomi.trianapp.data.model.UserProfileDto
import id.nomi.trianapp.data.remote.ApiClient
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.map

class GetProfileUseCase(
    private val apiClient: ApiClient,
    private val nomiDb: NomiDb
) {
    suspend fun fetchProfile(): Result<Unit> {
        return try {
            val response = apiClient.get<UserProfileDto>("/api/auth/profile")
            if (response.meta.code in 200..209 && response.data != null) {
                val dto = response.data
                val entity = ProfileEntity(
                    id = dto.id,
                    displayName = dto.displayName,
                    avatarUrl = dto.avatarUrl,
                    role = dto.role
                )
                nomiDb.nomiDao().insertProfile(entity)
                Result.success(Unit)
            } else {
                Result.failure(Exception(response.meta.message ?: "Failed to fetch profile"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    operator fun invoke(): Flow<ProfileEntity?> {
        return nomiDb.nomiDao().getProfile()
    }
}
