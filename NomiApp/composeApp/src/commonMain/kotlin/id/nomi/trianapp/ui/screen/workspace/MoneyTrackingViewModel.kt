package id.nomi.trianapp.ui.screen.workspace

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.local.MoneyTrackingEntity
import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.model.MoneyTransactionDto
import id.nomi.trianapp.data.model.TransactionItemDto
import id.nomi.trianapp.domain.usecase.GetMoneyHistoryUseCase
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json

sealed class MoneyTrackingState {
    object Idle : MoneyTrackingState()
    object Loading : MoneyTrackingState()
    data class Success(val items: List<MoneyTransactionDto>, val hasMore: Boolean) : MoneyTrackingState()
    data class Error(val message: String) : MoneyTrackingState()
}

class MoneyTrackingViewModel(
    private val getMoneyHistoryUseCase: GetMoneyHistoryUseCase
) : ViewModel() {
    private val _state = MutableStateFlow<MoneyTrackingState>(MoneyTrackingState.Idle)
    val state: StateFlow<MoneyTrackingState> = _state.asStateFlow()

    private var currentPage = 1
    private val allItems = mutableListOf<MoneyTransactionDto>()

    private val _selectedTransaction = MutableStateFlow<MoneyTransactionDto?>(null)
    val selectedTransaction: StateFlow<MoneyTransactionDto?> = _selectedTransaction.asStateFlow()

    init {
        observeLocalData()
        fetchHistory(isRefresh = true)
    }

    private fun observeLocalData() {
        viewModelScope.launch {
            getMoneyHistoryUseCase.getLocalHistory().collect { entities ->
                if (_state.value is MoneyTrackingState.Idle || _state.value is MoneyTrackingState.Error) {
                    val dtos = entities.map { it.toDto() }
                    if (dtos.isNotEmpty()) {
                        _state.value = MoneyTrackingState.Success(dtos, false)
                    }
                }
            }
        }
    }

    fun fetchHistory(isRefresh: Boolean = false) {
        if (isRefresh) {
            currentPage = 1
            allItems.clear()
        }

        viewModelScope.launch {
            if (allItems.isEmpty()) _state.value = MoneyTrackingState.Loading
            
            getMoneyHistoryUseCase(currentPage).onSuccess { response ->
                allItems.addAll(response.items)
                currentPage++
                val hasMore = allItems.size < response.totalCount
                _state.value = MoneyTrackingState.Success(allItems.toList(), hasMore)
            }.onFailure { e ->
                if (allItems.isEmpty()) {
                    _state.value = MoneyTrackingState.Error(e.message ?: "Unknown Error")
                }
            }
        }
    }

    fun selectTransaction(transaction: MoneyTransactionDto?) {
        _selectedTransaction.value = transaction
    }

    private fun MoneyTrackingEntity.toDto(): MoneyTransactionDto {
        return MoneyTransactionDto(
            id = id,
            merchantName = merchantName,
            category = category,
            description = description,
            totalAmount = totalAmount.toDouble(),
            createdAt = createdAt,
            items = try { Json.decodeFromString<List<TransactionItemDto>>(itemsJson) } catch (e: Exception) { emptyList() },
            userDisplayName = userDisplayName,
            conversationTitle = conversationTitle
        )
    }
}
