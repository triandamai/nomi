package id.nomi.trianapp.data.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class MoneyHistoryResponse(
    val items: List<MoneyTransactionDto>,
    @SerialName("total_count")
    val totalCount: Int
)

@Serializable
data class MoneyTransactionDto(
    val id: String,
    @SerialName("merchant_name")
    val merchantName: String,
    val category: String,
    val description: String? = null,
    @SerialName("total_amount")
    val totalAmount: Double,
    @SerialName("created_at")
    val createdAt: String,
    val items: List<TransactionItemDto>,
    @SerialName("user_display_name")
    val userDisplayName: String,
    @SerialName("conversation_title")
    val conversationTitle: String
)

@Serializable
data class TransactionItemDto(
    val name: String,
    val quantity: Int,
    @SerialName("total_amount")
    val totalAmount: Double
)
