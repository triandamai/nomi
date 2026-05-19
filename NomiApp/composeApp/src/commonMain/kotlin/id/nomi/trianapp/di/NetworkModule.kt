package id.nomi.trianapp.di

import id.nomi.trianapp.data.remote.ApiClient
import id.nomi.trianapp.data.remote.NomiMqttClient
import id.nomi.trianapp.data.remote.SseClient
import id.nomi.trianapp.util.EventBus
import io.ktor.client.*
import io.ktor.client.plugins.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.plugins.sse.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.serialization.json.Json
import org.koin.dsl.module

val networkModule = module {
    single { EventBus() }
    single { ApiClient(get()) }
    single { SseClient(get()) }
    single { NomiMqttClient(get()) }
}
