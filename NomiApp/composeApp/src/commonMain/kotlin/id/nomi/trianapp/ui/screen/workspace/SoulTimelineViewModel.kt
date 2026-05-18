package id.nomi.trianapp.ui.screen.workspace

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.model.SoulHistoryDto
import id.nomi.trianapp.data.preferences.PreferencesConstant
import id.nomi.trianapp.data.preferences.PreferencesStorage
import id.nomi.trianapp.domain.usecase.GetSoulHistoryUseCase
import id.nomi.trianapp.domain.usecase.RestoreSoulUseCase
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class SoulTimelineState(
    val isLoading: Boolean = false,
    val items: List<SoulHistoryDto> = emptyList(),
    val error: String? = null,
    val isRestoring: Boolean = false,
    val restoreSuccess: Boolean = false
)

class SoulTimelineViewModel(
    private val getSoulHistoryUseCase: GetSoulHistoryUseCase,
    private val restoreSoulUseCase: RestoreSoulUseCase,
    private val preferencesStorage: PreferencesStorage
) : ViewModel() {

    private val _uiState = MutableStateFlow(SoulTimelineState())
    val uiState: StateFlow<SoulTimelineState> = _uiState.asStateFlow()

    init {
        fetchSoulHistory()
    }

    fun fetchSoulHistory() {
        val conversationId = preferencesStorage.getString(PreferencesConstant.ACTIVE_CONVERSATION_ID)
        if (conversationId == null) {
            _uiState.update { it.copy(error = "No active conversation selected") }
            return
        }

        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            val response = getSoulHistoryUseCase(conversationId)
            if (response.data != null) {
                _uiState.update { it.copy(isLoading = false, items = response.data) }
            } else {
                _uiState.update { it.copy(isLoading = false, error = response.meta.message) }
            }
        }
    }

    fun restoreSoul(soulId: String) {
        val conversationId = preferencesStorage.getString(PreferencesConstant.ACTIVE_CONVERSATION_ID) ?: return

        viewModelScope.launch {
            _uiState.update { it.copy(isRestoring = true) }
            val response = restoreSoulUseCase(conversationId, soulId)
            if (response.meta.code == 200 || response.meta.code == 201) {
                _uiState.update { it.copy(isRestoring = false, restoreSuccess = true) }
                fetchSoulHistory()
            } else {
                _uiState.update { it.copy(isRestoring = false, error = response.meta.message) }
            }
        }
    }
    
    fun resetRestoreStatus() {
        _uiState.update { it.copy(restoreSuccess = false) }
    }
}
