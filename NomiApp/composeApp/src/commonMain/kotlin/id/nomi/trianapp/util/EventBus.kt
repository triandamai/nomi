package id.nomi.trianapp.util

import id.nomi.trianapp.data.model.MessageDto
import id.nomi.trianapp.data.model.MetadataDto
import id.nomi.trianapp.data.model.ThoughtDto
import id.nomi.trianapp.data.model.TokenUpdateDto
import id.nomi.trianapp.data.model.ToolEndDto
import id.nomi.trianapp.data.model.ToolStartDto
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow

class EventBus {
    private val _events = MutableSharedFlow<NomiEvent>(extraBufferCapacity = 64)
    val events: SharedFlow<NomiEvent> = _events.asSharedFlow()

    suspend fun emit(event: NomiEvent) {
        _events.emit(event)
    }

    fun tryEmit(event: NomiEvent) {
        _events.tryEmit(event)
    }
}

sealed class NomiEvent {
    data class Metadata(val data: MetadataDto) : NomiEvent()
    data class Thought(val data: ThoughtDto) : NomiEvent()
    data class Message(val data: MessageDto) : NomiEvent()
    data class Presence(val data: String) : NomiEvent()
    data class Typing(val data: String) : NomiEvent()
    data class TokenUpdate(val data: TokenUpdateDto) : NomiEvent()
    data class ToolStart(val data: ToolStartDto) : NomiEvent()
    data class ToolEnd(val data: ToolEndDto) : NomiEvent()
    data class Error(val message: String) : NomiEvent()
    data class Unknown(val type: String, val data: String) : NomiEvent()
}
