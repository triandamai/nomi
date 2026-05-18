package id.nomi.trianapp.data.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class ApiResponse<T>(
    val data: T? = null,
    val meta: MetaResponse,
    val errors: List<ErrorDetail>? = null
)

@Serializable
data class MetaResponse(
    val code: Int,
    val message: String
)

@Serializable
data class ErrorDetail(
    val name: String,
    @SerialName("error_message")
    val errorMessage: List<String>
)

