package id.nomi.trianapp.data.remote

import id.nomi.trianapp.data.preferences.PreferencesConstant
import id.nomi.trianapp.data.preferences.PreferencesStorage
import io.ktor.client.HttpClient
import io.ktor.client.engine.darwin.Darwin
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.defaultRequest
import io.ktor.client.request.header
import io.ktor.http.HttpHeaders
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json
import platform.Foundation.NSLocale
import platform.Foundation.currentLocale
import platform.Foundation.languageCode


import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.MetaResponse
import io.ktor.client.call.body
import io.ktor.client.engine.darwin.DarwinHttpRequestException
import io.ktor.client.plugins.sse.SSE
import io.ktor.client.statement.HttpResponse
import io.ktor.http.isSuccess

actual suspend inline fun <reified R> handleError(block: () -> HttpResponse): ApiResponse<R> {
    return try {
        val response = block()
        if (response.status.isSuccess()) {
            response.body()
        } else {
            try {
                response.body<ApiResponse<R>>()
            } catch (e: Exception) {
                ApiResponse(
                    meta = MetaResponse(
                        code = response.status.value,
                        message = response.status.description
                    )
                )
            }
        }
    } catch (e: DarwinHttpRequestException) {
        ApiResponse(
            meta = MetaResponse(
                code = e.origin.code.toInt(),
                message = e.message ?: "Darwin Network Error"
            )
        )
    } catch (e: Exception) {
        ApiResponse(
            meta = MetaResponse(
                code = 500,
                message = e.message ?: "Unknown error occurred"
            )
        )
    }
}

actual val baseUrl: String
    get() = "https://nomi-gateway.pakaiarta.id/api"


actual fun getHttpClient(
    preferences: PreferencesStorage
): HttpClient = HttpClient(Darwin) {
    defaultRequest {
        url()
        val header = "Bearer ${preferences.getString(PreferencesConstant.SESSION_TOKEN)}"
        val languageSetting = preferences.getString(PreferencesConstant.SESSION_LANGUAGE)
        val locale = if (languageSetting == null) {
            NSLocale.currentLocale.languageCode
        } else {
            when (languageSetting) {
                "en" -> "en"
                "id" -> "id"
                else -> NSLocale.currentLocale.languageCode
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
    engine {
        configureRequest {
            setAllowsCellularAccess(true)
        }
    }
    install(ContentNegotiation) {
        json(Json {
            prettyPrint = true
            isLenient = true
            ignoreUnknownKeys = true // Highly recommended to avoid crashes on new API fields
        })
    }
    install(SSE){
        this.maxReconnectionAttempts = 20
        this.showCommentEvents()
        this.showRetryEvents()
    }
}