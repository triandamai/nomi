package id.nomi.trianapp.data.remote

import io.ktor.client.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.sse.*
import io.ktor.client.request.*
import io.ktor.client.statement.bodyAsChannel
import io.ktor.http.HttpMethod
import io.ktor.sse.*
import io.ktor.utils.io.readUTF8Line
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.flow

class SseClient(private val client: HttpClient) {
    fun listenSseForDebug(url: String): Flow<String> = flow {
        client.prepareRequest {
            url(url)
            method = HttpMethod.Get
            headers.append("Accept", "text/event-stream")
        }.execute { response ->
            val channel = response.bodyAsChannel()
            while (!channel.isClosedForRead) {
                val line = channel.readUTF8Line()
                if (line != null) {
                    // 💡 THIS WILL FORCE EVERY SINGLE RAW BIT OF TEXT TO PRINT NO MATTER WHAT
                    println("🚨 RAW WIRE TEXT DROP: $line")
                    emit(line)
                }
            }
        }
    }.catch { e ->
        println("SSE Error: ${e.message}")
    }

    fun listenToSse(url: String): Flow<ServerSentEvent> = flow {
        client.sse(
            urlString = url,
            request = {
                // 💡 FORCE KT0R TO BYPASS NETWORK BUFFERS
                // This forces raw chunked transfer streaming to execute immediately
                headers.append("Accept", "text/event-stream")
                headers.append("Cache-Control", "no-cache")
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
                emit(event)
            }
        }
    }.catch { e ->
        println("SSE Error: ${e.message}")
    }
}
