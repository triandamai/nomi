package id.nomi.trianapp.ui.screen.workspace

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.local.ConversationEntity
import id.nomi.trianapp.data.local.ProfileEntity
import id.nomi.trianapp.domain.usecase.GetConversationsUseCase
import id.nomi.trianapp.domain.usecase.GetProfileUseCase
import id.nomi.trianapp.domain.usecase.SetActiveConversationUseCase
import id.nomi.trianapp.util.EventBus
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

sealed class WorkspaceState {
    object Idle : WorkspaceState()
    object Loading : WorkspaceState()
    object Success : WorkspaceState()
    data class Error(val message: String) : WorkspaceState()
}

class WorkspaceViewModel(
    private val getConversationsUseCase: GetConversationsUseCase,
    private val getProfileUseCase: GetProfileUseCase,
    private val setActiveConversationUseCase: SetActiveConversationUseCase,
    private val eventBus: EventBus
) : ViewModel() {

    private val _state = MutableStateFlow<WorkspaceState>(WorkspaceState.Idle)
    val state: StateFlow<WorkspaceState> = _state

    val conversations: StateFlow<List<ConversationEntity>> = getConversationsUseCase.getLocalConversations()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), emptyList())

    val profile: StateFlow<ProfileEntity?> = getProfileUseCase()
        .map { it.getOrNull() }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), null)

    init {
       viewModelScope.launch {
           eventBus.events.collect {
                println("WORKSPACE LISTENING ${it}")
           }
       }
    }
    fun fetchConversations() {
        viewModelScope.launch {
            _state.value = WorkspaceState.Loading
            try {
                val response = getConversationsUseCase()
                if (response.data != null) {
                    _state.value = WorkspaceState.Success
                } else {
                    _state.value = WorkspaceState.Error(response.meta.message)
                }
            } catch (e: Exception) {
                _state.value = WorkspaceState.Error(e.message ?: "Unknown error")
            }
        }
    }

    fun selectConversation(conversationId: String) {
        setActiveConversationUseCase(conversationId)
    }
}
