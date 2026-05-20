package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.preferences.PreferencesStorage
import id.nomi.trianapp.data.preferences.PreferencesConstant

class GetActiveConversationUseCase(
    private val preferencesStorage: PreferencesStorage
) {
    operator fun invoke(): String? {
        return preferencesStorage.getString(PreferencesConstant.ACTIVE_CONVERSATION_ID)
    }
}
