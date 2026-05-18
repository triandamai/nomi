package id.nomi.trianapp.ui.screen.workspace

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.local.ReminderEntity
import id.nomi.trianapp.data.model.ReminderDto
import id.nomi.trianapp.data.model.ReminderPayloadDto
import id.nomi.trianapp.domain.usecase.GetRemindersUseCase
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

sealed class RemindersState {
    object Idle : RemindersState()
    object Loading : RemindersState()
    data class Success(val reminders: List<ReminderDto>) : RemindersState()
    data class Error(val message: String) : RemindersState()
}

class RemindersViewModel(
    private val getRemindersUseCase: GetRemindersUseCase
) : ViewModel() {
    private val _state = MutableStateFlow<RemindersState>(RemindersState.Idle)
    val state: StateFlow<RemindersState> = _state.asStateFlow()

    init {
        observeLocalData()
        fetchReminders()
    }

    private fun observeLocalData() {
        viewModelScope.launch {
            getRemindersUseCase.getLocalReminders().collect { entities ->
                if (_state.value is RemindersState.Idle || _state.value is RemindersState.Error) {
                    val dtos = entities.map { it.toDto() }
                    if (dtos.isNotEmpty()) {
                        _state.value = RemindersState.Success(dtos)
                    }
                }
            }
        }
    }

    fun fetchReminders() {
        viewModelScope.launch {
            if (_state.value !is RemindersState.Success) {
                _state.value = RemindersState.Loading
            }
            try {
                val response = getRemindersUseCase()
                if (response.data != null) {
                    _state.value = RemindersState.Success(response.data)
                } else if (_state.value !is RemindersState.Success) {
                    _state.value = RemindersState.Error(response.meta.message)
                }
            } catch (e: Exception) {
                if (_state.value !is RemindersState.Success) {
                    _state.value = RemindersState.Error(e.message ?: "Unknown error")
                }
            }
        }
    }

    private fun ReminderEntity.toDto() = ReminderDto(
        id = id,
        taskType = taskType,
        payload = ReminderPayloadDto(content),
        content = content,
        dueAt = dueAt,
        frequency = frequency,
        status = status,
        userDisplayName = userDisplayName,
        conversationTitle = conversationTitle,
        createdAt = createdAt
    )
}
