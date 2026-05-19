package id.nomi.trianapp

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.model.MessageDto
import id.nomi.trianapp.data.model.MetadataDto
import id.nomi.trianapp.data.model.ThoughtDto
import id.nomi.trianapp.data.model.TokenUpdateDto
import id.nomi.trianapp.data.model.ToolEndDto
import id.nomi.trianapp.data.model.ToolStartDto
import id.nomi.trianapp.data.preferences.PreferencesConstant
import id.nomi.trianapp.data.preferences.PreferencesStorage
import id.nomi.trianapp.data.remote.NomiMqttClient
import id.nomi.trianapp.data.remote.SseClient
import id.nomi.trianapp.domain.usecase.GetProfileSyncUseCase
import id.nomi.trianapp.domain.usecase.LogoutUseCase
import id.nomi.trianapp.util.EventBus
import id.nomi.trianapp.util.NomiEvent
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

sealed class MainAppState {
    object Idle : MainAppState()
    object Authenticated : MainAppState()
    object Unauthenticated : MainAppState()
}

class MainViewModel(
    private val sseClient: SseClient,
    private val mqttClient: NomiMqttClient,
    private val preferencesStorage: PreferencesStorage,
    private val logoutUseCase: LogoutUseCase,
    private val getProfileSyncUseCase: GetProfileSyncUseCase,
    private val eventBus: EventBus
) : ViewModel() {

    private val _appState = MutableStateFlow<MainAppState>(MainAppState.Idle)
    val appState: StateFlow<MainAppState> = _appState

    init {
        checkAuthentication()
    }

    fun logout() {
        viewModelScope.launch {
            logoutUseCase()
            _appState.emit(MainAppState.Unauthenticated)
        }
    }

    @OptIn(ExperimentalUuidApi::class)
    fun checkAuthentication() = viewModelScope.launch {
        val token = preferencesStorage.getString(PreferencesConstant.SESSION_TOKEN)
        val u_id = preferencesStorage.getString(PreferencesConstant.ACTIVE_U_ID)
        if (!token.isNullOrEmpty()) {
            _appState.emit(MainAppState.Authenticated)
            if (u_id != null) {
                val existingDeviceId = preferencesStorage.getString(PreferencesConstant.ACTIVE_D_ID)
                var dId = Uuid.random().toString();
                if(existingDeviceId == null){
                    preferencesStorage.put(PreferencesConstant.ACTIVE_D_ID,dId)
                }
                if (existingDeviceId != null) {
                    dId = existingDeviceId
                }

                val uuid = Uuid.random();
                println("LISTEN user_id=${u_id} device_id=${dId}")
//                connect(u_id, dId)
                mqttClient.connect(u_id, dId)
            }
        } else {
            _appState.emit(MainAppState.Unauthenticated)
        }
    }

    fun connect(userId: String, deviceId: String) {
//        val sseJson = Json {
//            ignoreUnknownKeys = true
//            coerceInputValues = true
//            isLenient = true
//        }
//        viewModelScope.launch {
//            sseClient.listenToSse("/api/realtime?user_id=$userId&device_id=$deviceId")
//                .catch {
//                    eventBus.emit(NomiEvent.Error(it.message ?: "Unknown SSE Error"))
//                }
//                .collect { event ->
//                    val nomiEvent = when (event.event) {
//                        "metadata" -> {
//                            if (event.data != null) {
//                                val data = sseJson.decodeFromString<MetadataDto>(event.data ?: "{}");
//                                NomiEvent.Metadata(data)
//                            } else {
//                                NomiEvent.Error("failed parsing")
//                            }
//                        }
//
//                        "thought" -> {
//                            if (event.data != null) {
//                                val data = sseJson.decodeFromString<ThoughtDto>(event.data ?: "{}");
//                                NomiEvent.Thought(data)
//                            } else {
//                                NomiEvent.Error("failed parsing")
//                            }
//                        }
//
//                        "message" -> {
//                            if (event.data != null) {
//                                val data = sseJson.decodeFromString<MessageDto>(event.data ?: "{}");
//                                NomiEvent.Message(data)
//                            } else {
//                                NomiEvent.Error("failed parsing")
//                            }
//                        }
//
//                        "presence" -> {
//                            NomiEvent.Presence(event.data ?: "")
//                        }
//
//                        "typing" -> {
//                            NomiEvent.Typing(event.data ?: "")
//                        }
//
//                        "token_update" -> {
//                            if (event.data != null) {
//                                val data =
//                                    sseJson.decodeFromString<TokenUpdateDto>(event.data ?: "{}");
//                                NomiEvent.TokenUpdate(data)
//                            } else {
//                                NomiEvent.Error("failed parsing")
//                            }
//                        }
//
//                        "tool_start" -> {
//                            if (event.data != null) {
//                                val data = sseJson.decodeFromString<ToolStartDto>(event.data ?: "{}");
//                                NomiEvent.ToolStart(data)
//                            } else {
//                                NomiEvent.Error("failed parsing")
//                            }
//                        }
//
//                        "tool_end" -> {
//                            if (event.data != null) {
//                                val data = sseJson.decodeFromString<ToolEndDto>(event.data ?: "{}");
//                                NomiEvent.ToolEnd(data)
//                            } else {
//                                NomiEvent.Error("failed parsing")
//                            }
//                        }
//
//                        else -> {
//                            NomiEvent.Unknown(event.event ?: "unknown", event.data ?: "")
//                        }
//                    }
//                    eventBus.emit(nomiEvent)
//                }
//        }
    }
}
