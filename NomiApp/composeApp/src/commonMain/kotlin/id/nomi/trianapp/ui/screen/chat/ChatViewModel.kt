package id.nomi.trianapp.ui.screen.chat

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.local.MessageEntity
import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.model.MessageDto
import id.nomi.trianapp.domain.usecase.FetchMessagesUseCase
import id.nomi.trianapp.domain.usecase.GetActiveConversationUseCase
import id.nomi.trianapp.util.EventBus
import id.nomi.trianapp.util.NomiEvent
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.datetime.Clock

class ChatViewModel(
    private val fetchMessagesUseCase: FetchMessagesUseCase,
    private val getActiveConversationUseCase: GetActiveConversationUseCase,
    private val nomiDb: NomiDb,
    private val eventBus: EventBus
) : ViewModel() {

    private val _activeConversationId = MutableStateFlow<String?>(null)
    val activeConversationId: StateFlow<String?> = _activeConversationId

    private val _messages = MutableStateFlow<List<MessageEntity>>(emptyList())
    val messages: StateFlow<List<MessageEntity>> = _messages

    private val _thought = MutableStateFlow<String?>(null)
    val thought: StateFlow<String?> = _thought

    private val _activeTool = MutableStateFlow<String?>(null)
    val activeTool: StateFlow<String?> = _activeTool

    private val _isTyping = MutableStateFlow(false)
    val isTyping: StateFlow<Boolean> = _isTyping

    init {
        _activeConversationId.value = getActiveConversationUseCase()

        viewModelScope.launch {
            _activeConversationId.collect { id ->
                id?.let {
                    fetchMessages(it)
                    observeLocalMessages(it)
                }
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

    private fun fetchMessages(conversationId: String) {
        viewModelScope.launch {
            fetchMessagesUseCase(conversationId)
        }
    }

    private fun observeLocalMessages(conversationId: String) {
        viewModelScope.launch {
            fetchMessagesUseCase.getLocalMessages(conversationId).collect {
                _messages.value = it
            }
        }
    }

    private suspend fun handleNewMessage(dto: MessageDto) {
        val conversationId = _activeConversationId.value ?: return
        if (dto.conversationId != conversationId) {
            return
        }
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
        nomiDb.nomiDao().insertMessages(listOf(newMessage))
        _thought.value = null
    }
}
