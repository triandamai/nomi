package id.nomi.trianapp.data.remote

import id.nomi.trianapp.data.model.*
import id.nomi.trianapp.util.EventBus
import id.nomi.trianapp.util.NomiEvent
import io.github.davidefioravanti.kmqtt.client.MqttClient
import io.github.davidefioravanti.kmqtt.client.MqttClientConfig
import io.github.davidefioravanti.kmqtt.client.MqttMessage
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.IO
import kotlinx.coroutines.launch
import kotlinx.serialization.json.Json

class NomiMqttClient(
    private val eventBus: EventBus
) {
    private var client: MqttClient? = null
    private val json = Json {
        ignoreUnknownKeys = true
        coerceInputValues = true
        isLenient = true
    }
    private val scope = CoroutineScope(Dispatchers.IO)

    fun connect(userId: String) {
        if (client != null) return

        // Using a hierarchical client ID allows EMQX ACLs to use %c wildcards easily
        val uniqueClientId = "nomi/users/$userId/mobile"

        val config = MqttClientConfig(
            brokerHost = "b1fec516.ala.eu-central-1.emqxsl.com",
            brokerPort = 8084,
            clientId = uniqueClientId,
            userName = "nomi-client-app",
            password = "NomiPublicPass2026",
            isTls = true,
            isWss = true,
            path = "/mqtt",
            cleanSession = false // Broker remembers subscriptions
        )

        client = MqttClient(config)

        client?.onMessageArrived = { topic, message ->
            handleMessage(topic, message)
        }

        client?.onConnected = {
            println("MQTT: Connected as $userId")
            subscribe(userId)
        }

        client?.onConnectionFailed = {
            println("MQTT: Connection failed")
        }

        client?.connect()
    }

    private fun subscribe(userId: String) {
        client?.subscribe("nomi/users/$userId/#")
        client?.subscribe("nomi/broadcast/#")
    }

    fun setConversation(conversationId: String) {
        client?.subscribe("nomi/conversations/$conversationId/#")
    }

    private fun handleMessage(topic: String, message: MqttMessage) {
        val payload = message.payload.decodeToString()
        val parts = topic.split("/")
        val eventName = parts.last()

        scope.launch {
            val nomiEvent = when (eventName) {
                "message" -> {
                    try {
                        val data = json.decodeFromString<MessageDto>(payload)
                        NomiEvent.Message(data)
                    } catch (e: Exception) {
                        NomiEvent.Error("failed parsing MQTT message")
                    }
                }
                "thought" -> {
                    try {
                        val data = json.decodeFromString<ThoughtDto>(payload)
                        NomiEvent.Thought(data)
                    } catch (e: Exception) {
                        NomiEvent.Error("failed parsing MQTT thought")
                    }
                }
                "token_update" -> {
                    try {
                        val data = json.decodeFromString<TokenUpdateDto>(payload)
                        NomiEvent.TokenUpdate(data)
                    } catch (e: Exception) {
                        NomiEvent.Error("failed parsing MQTT token_update")
                    }
                }
                "presence" -> NomiEvent.Presence(payload)
                "typing" -> NomiEvent.Typing(payload)
                else -> NomiEvent.Unknown(eventName, payload)
            }
            eventBus.emit(nomiEvent)
        }
    }

    fun disconnect() {
        client?.disconnect()
        client = null
    }
}
