package id.nomi.trianapp.ui.screen.workspace

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.model.StorageItemDto
import id.nomi.trianapp.domain.usecase.DeleteStorageFileUseCase
import id.nomi.trianapp.domain.usecase.ExploreStorageUseCase
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class StorageManagementState(
    val isLoading: Boolean = false,
    val items: List<StorageItemDto> = emptyList(),
    val currentPath: String = "",
    val error: String? = null,
    val isDeleting: Boolean = false,
    val pathHistory: List<String> = emptyList()
)

class StorageManagementViewModel(
    private val exploreStorageUseCase: ExploreStorageUseCase,
    private val deleteStorageFileUseCase: DeleteStorageFileUseCase
) : ViewModel() {

    private val _uiState = MutableStateFlow(StorageManagementState())
    val uiState: StateFlow<StorageManagementState> = _uiState.asStateFlow()

    init {
        explore()
    }

    fun explore(prefix: String? = null) {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            val response = exploreStorageUseCase(prefix)
            if (response.data != null) {
                _uiState.update { state ->
                    val newHistory = if (prefix == null) emptyList() 
                                     else if (prefix == "" && state.currentPath != "") emptyList()
                                     else if (prefix.length < state.currentPath.length) state.pathHistory.dropLast(1)
                                     else if (prefix != state.currentPath) state.pathHistory + prefix
                                     else state.pathHistory
                    
                    state.copy(
                        isLoading = false, 
                        items = response.data, 
                        currentPath = prefix ?: "",
                        pathHistory = newHistory
                    )
                }
            } else {
                _uiState.update { it.copy(isLoading = false, error = response.meta.message) }
            }
        }
    }

    fun navigateBack() {
        val history = _uiState.value.pathHistory
        if (history.size > 1) {
            val previousPath = history[history.size - 2]
            explore(previousPath)
        } else {
            explore(null)
        }
    }

    fun deleteFile(item: StorageItemDto) {
        viewModelScope.launch {
            _uiState.update { it.copy(isDeleting = true) }
            val response = deleteStorageFileUseCase(item.fullPath)
            if (response.meta.code in 200..204) {
                explore(_uiState.value.currentPath.ifEmpty { null })
            } else {
                _uiState.update { it.copy(isDeleting = false, error = response.meta.message) }
            }
        }
    }
}
