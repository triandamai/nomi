package id.nomi.trianapp.ui.screen.workspace

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.model.ConversationDto
import id.nomi.trianapp.domain.usecase.GetConversationsUseCase
import id.nomi.trianapp.domain.usecase.UpdateConversationUseCase
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class ConversationMonitoringState(
    val isLoading: Boolean = false,
    val conversations: List<ConversationDto> = emptyList(),
    val error: String? = null,
    val isUpdating: Boolean = false
)

class ConversationMonitoringViewModel(
    private val getConversationsUseCase: GetConversationsUseCase,
    private val updateConversationUseCase: UpdateConversationUseCase
) : ViewModel() {

    private val _uiState = MutableStateFlow(ConversationMonitoringState())
    val uiState: StateFlow<ConversationMonitoringState> = _uiState.asStateFlow()

    init {
        fetchConversations()
    }

    fun fetchConversations() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            val response = getConversationsUseCase()
            if (response.data != null) {
                _uiState.update { it.copy(isLoading = false, conversations = response.data) }
            } else {
                _uiState.update { it.copy(isLoading = false, error = response.meta.message) }
            }
        }
    }

    fun updateMaxTokens(conversationId: String, maxTokenUsage: Long) {
        viewModelScope.launch {
            _uiState.update { it.copy(isUpdating = true) }
            val response = updateConversationUseCase(conversationId, maxTokenUsage)
            if (response.data != null) {
                _uiState.update { state ->
                    val updatedList = state.conversations.map { 
                        if (it.id == conversationId) response.data else it 
                    }
                    state.copy(isUpdating = false, conversations = updatedList)
                }
            } else {
                _uiState.update { it.copy(isUpdating = false, error = response.meta.message) }
            }
        }
    }
}
