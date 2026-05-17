package id.nomi.trianapp.data.remote

import io.ktor.client.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.sse.*
import io.ktor.client.request.*
import io.ktor.sse.*
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.flow

class SseClient(private val client: HttpClient) {
    fun listenToSse(url: String): Flow<ServerSentEvent> = flow {
        client.sse(
            urlString = url,
            request = {
                timeout {
                    // Use -1 or 0 for infinite timeout depending on engine, 
                    // but Ktor generally uses Long.MAX_VALUE for infinite in some contexts,
                    // or literal values if constant is unresolved.
                    requestTimeoutMillis = HttpTimeoutConfig.INFINITE_TIMEOUT_MS // Disable timeout
                    socketTimeoutMillis = HttpTimeoutConfig.INFINITE_TIMEOUT_MS  // Disable timeout
                }
            }
        ) {
            incoming.collect { event ->
                println("Event: $event}")
                emit(event)
            }
        }
    }.catch { e ->
        println("SSE Error: ${e.message}")
    }
}
