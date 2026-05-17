package id.nomi.trianapp

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.preferences.PreferencesConstant
import id.nomi.trianapp.data.preferences.PreferencesStorage
import id.nomi.trianapp.data.remote.SseClient
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.launch

sealed class MainAppState {
    object Idle : MainAppState()
    object Authenticated : MainAppState()
    object Unauthenticated : MainAppState()
}

class MainViewModel(
    private val sseClient: SseClient,
    private val preferencesStorage: PreferencesStorage
) : ViewModel() {

    private val _appState = MutableStateFlow<MainAppState>(MainAppState.Idle)
    val appState: StateFlow<MainAppState> = _appState

    init {
        checkAuthentication()
    }

    fun checkAuthentication() = viewModelScope.launch{
        val token = preferencesStorage.getString(PreferencesConstant.SESSION_TOKEN)
        if (!token.isNullOrEmpty()) {
            _appState.emit(MainAppState.Authenticated)
            connect("","")
        } else {
            _appState.emit(MainAppState.Unauthenticated)
        }
    }

    fun connect(userId: String, deviceId: String) {
        viewModelScope.launch {
            println("START SSE LISTEN")
            sseClient.listenToSse("/api/realtime?user_id=$userId&device_id=$deviceId")
                .catch {
                    println("SSE ERROR ${it.message}")
                }
                .collect { event ->
                    println("Received: ${event.data}")
                }
        }
    }
}