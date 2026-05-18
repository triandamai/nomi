package id.nomi.trianapp.data.remote

import id.nomi.trianapp.data.preferences.PreferencesConstant
import id.nomi.trianapp.data.preferences.PreferencesStorage
import io.ktor.client.HttpClient
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.request.header
import io.ktor.http.HttpHeaders
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json
import java.util.Locale

import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.MetaResponse
import io.ktor.client.call.body
import io.ktor.client.plugins.sse.SSE
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.isSuccess
import kotlin.time.Duration

actual suspend inline fun <reified R> handleError(block: () -> HttpResponse): ApiResponse<R> {
    return try {
        val response = block()
        if (response.status.isSuccess()) {
            println("${response.bodyAsText()}")
            response.body()
        } else {
            try {
                if (response.status.value >= 200 && response.status.value <= 209) {
                    response.body<ApiResponse<R>>()
                }else{
                    response.body<ApiResponse<R>>()
                }
            } catch (e: Exception) {
                println(e.message)
                ApiResponse(
                    meta = MetaResponse(
                        code = response.status.value,
                        message = response.status.description
                    )
                )
            }
        }
    } catch (e: Exception) {
        ApiResponse(
            meta = MetaResponse(
                code = 500,
                message = e.message ?: "Unknown error occurred"
            )
        )
    }
}

actual fun getHttpClient(
    preferences: PreferencesStorage
): HttpClient =
    HttpClient(OkHttp) {
        defaultRequest {
            url(baseUrl)
            val header = "Bearer ${preferences.getString(PreferencesConstant.SESSION_TOKEN)}"
            val languageSetting = preferences.getString(PreferencesConstant.SESSION_LANGUAGE)
            val locale = if (languageSetting == null) {
                Locale.getDefault().toLanguageTag().split("-")[0]
            } else {
                when (languageSetting) {
                    "en" -> "en"
                    "id" -> "id"
                    else -> Locale.getDefault().toLanguageTag().split("-")[0]
                }
            }
            header(
                HttpHeaders.AcceptLanguage,
                locale
            )

            header(HttpHeaders.Authorization, header)
            header(
                HttpHeaders.UserAgent,
                "${preferences.getDeviceId()};${preferences.getDeviceName()};${preferences.getDeviceType()}"
            )
        }
        install(ContentNegotiation) {
            json(Json {
                prettyPrint = true
                isLenient = true
                ignoreUnknownKeys = true // Highly recommended to avoid crashes on new API fields
//                coerceInputValues = true
            })
        }
        install(SSE) {
            this.maxReconnectionAttempts = 20
            this.showCommentEvents()
            this.showRetryEvents()
        }
    }

actual val baseUrl: String
    get() = "https://nomi-gateway.pakaiarta.id"
//    get() = "http://localhost:8000"