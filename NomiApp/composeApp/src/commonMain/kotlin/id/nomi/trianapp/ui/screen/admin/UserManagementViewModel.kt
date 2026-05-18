package id.nomi.trianapp.ui.screen.admin

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.model.AdminUserDetailResponse
import id.nomi.trianapp.data.model.AdminUserDto
import id.nomi.trianapp.domain.usecase.GetAdminUserDetailUseCase
import id.nomi.trianapp.domain.usecase.GetAdminUsersUseCase
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class UserManagementState(
    val isLoading: Boolean = false,
    val users: List<AdminUserDto> = emptyList(),
    val selectedUserDetail: AdminUserDetailResponse? = null,
    val isDetailLoading: Boolean = false,
    val error: String? = null
)

class UserManagementViewModel(
    private val getAdminUsersUseCase: GetAdminUsersUseCase,
    private val getAdminUserDetailUseCase: GetAdminUserDetailUseCase
) : ViewModel() {

    private val _uiState = MutableStateFlow(UserManagementState())
    val uiState: StateFlow<UserManagementState> = _uiState.asStateFlow()

    init {
        fetchUsers()
    }

    fun fetchUsers() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            val response = getAdminUsersUseCase()
            if (response.data != null) {
                _uiState.update { it.copy(isLoading = false, users = response.data.items) }
            } else {
                _uiState.update { it.copy(isLoading = false, error = response.meta.message) }
            }
        }
    }

    fun fetchUserDetail(userId: String) {
        viewModelScope.launch {
            _uiState.update { it.copy(isDetailLoading = true, error = null, selectedUserDetail = null) }
            val response = getAdminUserDetailUseCase(userId)
            if (response.data != null) {
                _uiState.update { it.copy(isDetailLoading = false, selectedUserDetail = response.data) }
            } else {
                _uiState.update { it.copy(isDetailLoading = false, error = response.meta.message) }
            }
        }
    }
    
    fun clearSelectedUser() {
        _uiState.update { it.copy(selectedUserDetail = null) }
    }
}
