package id.nomi.trianapp.ui.screen.chat

import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.local.ConversationEntity
import id.nomi.trianapp.data.local.MessageEntity
import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.local.ProfileEntity
import id.nomi.trianapp.data.model.MessageDto
import id.nomi.trianapp.domain.usecase.*
import id.nomi.trianapp.util.EventBus
import id.nomi.trianapp.util.NomiEvent
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.datetime.Clock

class ChatViewModel(
    private val fetchMessagesUseCase: FetchMessagesUseCase,
    private val getConversationUseCase: GetConversationUseCase,
    private val getProfileUseCase: GetProfileUseCase,
    private val sendMessageUseCase: SendMessageUseCase,
    private val nomiDb: NomiDb,
    private val eventBus: EventBus
) : ViewModel() {


    private val _activeConversation = MutableStateFlow<ConversationEntity?>(null)
    val activeConversation: StateFlow<ConversationEntity?> = _activeConversation

    private val _profile = MutableStateFlow<ProfileEntity?>(null)
    val profile: StateFlow<ProfileEntity?> = _profile

    val messages: SnapshotStateList<MessageEntity> = SnapshotStateList()

    private val _thought = MutableStateFlow<String?>(null)
    val thought: StateFlow<String?> = _thought

    private val _activeTool = MutableStateFlow<String?>(null)
    val activeTool: StateFlow<String?> = _activeTool

    private val _isTyping = MutableStateFlow(false)
    val isTyping: StateFlow<Boolean> = _isTyping

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading

    private var conversationId: String? = null

    fun setConversationId(id: String) {
        conversationId = id
        viewModelScope.launch {
            _isLoading.value = true
            fetchConversation(id)
            observeLocalConversation(id)
            fetchMessages(id)
            observeLocalMessages(id)
            _isLoading.value = false
        }
    }

    fun resetConversation() {
        _activeConversation.value = null
        messages.clear()
        _thought.value = null
        _activeTool.value = null
        _isTyping.value = false
        _isLoading.value = false
    }

    init {
        viewModelScope.launch {
            getProfileUseCase.fetchProfile()
        }

        viewModelScope.launch {
            getProfileUseCase().collect {
                _profile.value = it
            }
        }



        viewModelScope.launch {
            eventBus.events.collect { event ->
                when (event) {
                    is NomiEvent.Message -> {
                        _isTyping.value = false
                        handleNewMessage(event.data)
                    }

                    is NomiEvent.Thought -> {
                        _thought.value = event.data.text
                    }

                    is NomiEvent.ToolStart -> {
                        _activeTool.value = event.data.name
                    }

                    is NomiEvent.ToolEnd -> {
                        _activeTool.value = null
                    }

                    is NomiEvent.Presence -> {
                        _isTyping.value = true
                    }

                    is NomiEvent.Typing -> {
                        _isTyping.value = true
                    }

                    else -> {}
                }
            }
        }
    }

    private fun fetchConversation(id: String) {
        viewModelScope.launch {
            getConversationUseCase(id)
        }
    }


    private fun observeLocalConversation(id: String) {
        viewModelScope.launch {
            getConversationUseCase.getLocalConversation(id).collect {
                _activeConversation.value = it
            }
        }
    }

    fun sendMessage(content: String) {
        viewModelScope.launch {
            val id = conversationId
            if (id != null) {
                sendMessageUseCase(id, content)
            }
        }
    }

    private fun fetchMessages(conversationId: String) {
        viewModelScope.launch {
            fetchMessagesUseCase(conversationId)
        }
    }

    private fun observeLocalMessages(conversationId: String) {
        viewModelScope.launch {
            fetchMessagesUseCase.getLocalMessages(conversationId).onEach { msgs ->
                msgs.forEach { new ->
                    val find = messages.withIndex().find { it.value.id == new.id }
                    if (find == null) {
                        messages.add(new)
                    } else {
                        messages[find.index] = new
                    }
                }
            }.catch {
                messages.clear()
            }.collect()
        }
    }

    private suspend fun handleNewMessage(dto: MessageDto) {
        val newMessage = MessageEntity(
            id = dto.id,
            conversationId = dto.conversationId,
            displayName = dto.displayName ?: "",
            role = dto.role,
            content = dto.content,
            totalTokens = dto.totalTokens ?: 0,
            answerTokens = dto.answerTokens ?: 0,
            promptTokens = dto.promptTokens ?: 0,
            thought = dto.thought,
            imageUrl = dto.imageUrl,
            videoUrl = dto.videoUrl,
            audioUrl = dto.audioUrl,
            documentUrl = dto.documentUrl,
            stickerUrl = dto.stickerUrl,
            userId = dto.userId,
            createdAt = dto.createdAt
        )
        if (newMessage.conversationId == conversationId) {
            val find = messages.withIndex().find { it.value.id == newMessage.id }
            if (find == null) {
                messages.add(newMessage)
            } else {
                messages[find.index] = newMessage
            }
        }
        nomiDb.nomiDao().insertMessages(listOf(newMessage))
        _thought.value = null
    }
}
