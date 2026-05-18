package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.preferences.PreferencesStorage
import id.nomi.trianapp.data.preferences.PreferencesConstant

class SetActiveConversationUseCase(
    private val preferencesStorage: PreferencesStorage
) {
    operator fun invoke(conversationId: String) {
        preferencesStorage.put(PreferencesConstant.ACTIVE_CONVERSATION_ID, conversationId)
    }
}
