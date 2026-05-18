package id.nomi.trianapp.ui.screen.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.PairingDto
import id.nomi.trianapp.domain.usecase.SendPairingRequestUseCase
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

sealed class LoginState {
    object Idle : LoginState()
    object Loading : LoginState()
    data class Success(val data: PairingDto) : LoginState()
    data class Error(val message: String) : LoginState()
}

class LoginViewModel(
    private val sendPairingRequestUseCase: SendPairingRequestUseCase
) : ViewModel() {

    private val _loginState = MutableStateFlow<LoginState>(LoginState.Idle)
    val loginState: StateFlow<LoginState> = _loginState

    fun sendPairingRequest(pairingCode: String) {
        viewModelScope.launch {
            _loginState.value = LoginState.Loading
            try {
                val response: ApiResponse<PairingDto> = sendPairingRequestUseCase(pairingCode)
                if (response.data != null) {
                    _loginState.value = LoginState.Success(response.data)
                } else {
                    val errorMessage = response.errors?.map { it.errorMessage.joinToString(",") }?.joinToString(",")
                        ?: response.meta.message
                    _loginState.value = LoginState.Error(errorMessage)
                }
            } catch (e: Exception) {
                _loginState.value = LoginState.Error(e.message ?: "Unknown error occurred")
            }
        }
    }
}
