package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.local.MoneyTrackingEntity
import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.model.MoneyHistoryResponse
import id.nomi.trianapp.data.remote.ApiClient
import kotlinx.coroutines.flow.Flow
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

class GetMoneyHistoryUseCase(
    private val apiClient: ApiClient,
    private val nomiDb: NomiDb
) {
    suspend operator fun invoke(page: Int = 1): Result<MoneyHistoryResponse> {
        return try {
            val response = apiClient.get<MoneyHistoryResponse>("/api/v1/money/history?page=$page")

            if (response.data != null) {
                val entities = response.data.items.map { dto ->
                    MoneyTrackingEntity(
                        id = dto.id,
                        merchantName = dto.merchantName,
                        category = dto.category,
                        description = dto.description,
                        totalAmount = dto.totalAmount.toLong(),
                        createdAt = dto.createdAt,
                        itemsJson = Json.encodeToString(dto.items),
                        userDisplayName = dto.userDisplayName,
                        conversationTitle = dto.conversationTitle
                    )
                }
                if (page == 1) {
                    nomiDb.nomiDao().deleteMoneyTracking()
                }
                nomiDb.nomiDao().insertMoneyTracking(entities)
                Result.success(response.data)
            } else {
                Result.failure(Exception(response.meta.message))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    fun getLocalHistory(): Flow<List<MoneyTrackingEntity>> {
        return nomiDb.nomiDao().getMoneyTracking()
    }
}
