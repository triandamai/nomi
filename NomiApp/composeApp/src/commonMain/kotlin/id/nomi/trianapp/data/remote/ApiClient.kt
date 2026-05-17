package id.nomi.trianapp.data.remote

import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.MetaResponse
import id.nomi.trianapp.data.preferences.PreferencesStorage
import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.*
import io.ktor.client.request.forms.*
import io.ktor.client.statement.HttpResponse
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json

expect val baseUrl: String
expect fun getHttpClient(
    preferences: PreferencesStorage
): HttpClient

expect suspend inline fun <reified R> handleError(block: () -> HttpResponse): ApiResponse<R>

class ApiClient(val client: HttpClient) {

    suspend inline fun <reified T, reified R> post(url: String, body: T): ApiResponse<R> =
        handleError {
            client.post(url) {
                contentType(ContentType.Application.Json)
                setBody(body)
            }
        }

    suspend inline fun <reified R> get(url: String): ApiResponse<R> =
        handleError {
            client.get(url)
        }

    suspend inline fun <reified T, reified R> put(url: String, body: T): ApiResponse<R> =
        handleError {
            client.put(url) {
                contentType(ContentType.Application.Json)
                setBody(body)
            }
        }

    suspend inline fun <reified T, reified R> patch(url: String, body: T): ApiResponse<R> =
        handleError {
            client.patch(url) {
                contentType(ContentType.Application.Json)
                setBody(body)
            }
        }

    suspend inline fun <reified R> delete(url: String): ApiResponse<R> =
        handleError {
            client.delete(url)
        }

    suspend inline fun <reified R> postForm(url: String, formData: Parameters): ApiResponse<R> =
        handleError {
            client.submitForm(url = url, formParameters = formData)
        }

    suspend inline fun <reified R> putForm(url: String, formData: Parameters): ApiResponse<R> =
        handleError {
            client.submitForm(url = url, formParameters = formData) {
                method = HttpMethod.Put
            }
        }

    suspend inline fun <reified R> patchForm(url: String, formData: Parameters): ApiResponse<R> =
        handleError {
            client.submitForm(url = url, formParameters = formData) {
                method = HttpMethod.Patch
            }
        }
}
