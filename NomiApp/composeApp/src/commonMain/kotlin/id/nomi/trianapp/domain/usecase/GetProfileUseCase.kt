package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.local.ProfileEntity
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.map

class GetProfileUseCase(
    private val nomiDb: NomiDb
) {
    /**
     * Fetches, validates, and returns the local user profile.
     * Handles potential database read errors safely by wrapping the returned stream in a Result.
     */
    operator fun invoke(): Flow<Result<ProfileEntity?>> {
        return nomiDb.nomiDao().getProfile()
            .map { profile ->
                Result.success(profile)
            }
            .catch { e ->
                emit(Result.failure(e))
            }
    }
}
