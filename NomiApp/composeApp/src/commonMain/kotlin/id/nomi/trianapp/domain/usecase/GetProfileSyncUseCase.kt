package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.local.ProfileEntity

class GetProfileSyncUseCase(
    private val nomiDb: NomiDb
) {
    /**
     * Fetches the local user profile once (non-streaming).
     * Returns a Result containing the profile or an error.
     */
    suspend operator fun invoke(): ProfileEntity? = nomiDb.nomiDao().getProfileSync()

}
