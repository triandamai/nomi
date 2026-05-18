package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.preferences.PreferencesConstant
import id.nomi.trianapp.data.preferences.PreferencesStorage

class LogoutUseCase(
    private val preferencesStorage: PreferencesStorage,
    private val nomiDb: NomiDb
) {
    /**
     * Clears user session token and deletes local database data.
     */
    suspend operator fun invoke(): Result<Unit> {
        return try {
            // Clear session token
            preferencesStorage.put(PreferencesConstant.SESSION_TOKEN, "")
            
            // Clear local database
            nomiDb.nomiDao().deleteConversations()
            nomiDb.nomiDao().deleteChannels()
            nomiDb.nomiDao().deleteProfile()
            
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
