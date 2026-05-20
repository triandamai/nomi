package id.nomi.trianapp.data.remote

import id.nomi.trianapp.data.model.*
import id.nomi.trianapp.util.EventBus
import id.nomi.trianapp.util.NomiEvent
import io.github.davidepianca98.MQTTClient
import io.github.davidepianca98.mqtt.MQTTVersion
import io.github.davidepianca98.mqtt.Subscription
import io.github.davidepianca98.mqtt.packets.Qos
import io.github.davidepianca98.mqtt.packets.mqttv5.ReasonCode
import io.github.davidepianca98.mqtt.packets.mqttv5.SubscriptionOptions
import io.github.davidepianca98.socket.tls.TLSClientSettings
import io.ktor.utils.io.core.toByteArray
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.IO
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json

class NomiMqttClient(
    private val eventBus: EventBus
) {
    private var client: MQTTClient? = null

    private val json = Json {
        ignoreUnknownKeys = true
        coerceInputValues = true
        isLenient = true
    }
    private val scope = CoroutineScope(Dispatchers.IO)
    private var currentUserId: String? = null

    @OptIn(ExperimentalUnsignedTypes::class)
    fun connect(userId: String, deviceId: String) {
        if (client != null) return
        this.currentUserId = userId

        val uniqueClientId = "nomi/users/$userId/mobile_$deviceId"

        scope.launch {
            while (currentUserId != null) { // Simple reconnection loop
                try {
                    withContext(Dispatchers.IO) {
                        val setting = TLSClientSettings()
                        
                        val cl = MQTTClient(
                            MQTTVersion.MQTT3_1_1,
                            "b1fec516.ala.eu-central-1.emqxsl.com",
                            8084,
                            setting,
                            webSocket = "/mqtt",
                            userName = "nomi-client-app",
                            password = "NomiPublicPass2026".toByteArray().toUByteArray()
                        ) { message ->
                            handleMessage(message.topicName, message.payload?.toByteArray()?.decodeToString() ?: "")
                        }

                        client = cl
                        println("MQTT: Connected to $uniqueClientId")
                        
                        // Subscribe before running the loop or in a separate launch
                        launch {
                            delay(1000) // Small delay to ensure connection is established
                            subscribe()
                        }

                        cl.run() // This blocks until disconnected
                    }
                } catch (e: Exception) {
                    println("MQTT: Connection lost or failed: ${e.message}")
                } finally {
                    client = null
                }
                
                if (currentUserId != null) {
                    println("MQTT: Reconnecting in 5 seconds...")
                    delay(5000)
                }
            }
        }
    }

    private suspend fun subscribe() {
      withContext(Dispatchers.IO){
          val subscriptions = listOf(
              Subscription("nomi/users/$currentUserId/#", SubscriptionOptions(Qos.AT_LEAST_ONCE)),
              Subscription("nomi/broadcast/#", SubscriptionOptions(Qos.AT_LEAST_ONCE))
          )
          client?.subscribe(subscriptions)
      }
    }

    suspend fun setConversation(conversationId: String) {
        if (currentUserId == null){
            println("USER ID IS NULL")
            return
        }
        withContext(Dispatchers.IO) {
            val subscriptions = listOf(
                Subscription("nomi/users/$currentUserId/#", SubscriptionOptions(Qos.AT_LEAST_ONCE)),
                Subscription("nomi/broadcast/#", SubscriptionOptions(Qos.AT_LEAST_ONCE)),
                Subscription(
                    "nomi/conversations/$conversationId/#",
                    SubscriptionOptions(Qos.AT_LEAST_ONCE)
                )
            )

            client?.subscribe(subscriptions)
        }
    }

    private fun handleMessage(topic: String, payload: String) {
        println("EVENT ${topic} payload: ${payload}")
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
        client?.disconnect(reasonCode = ReasonCode.RE_AUTHENTICATE)
        client = null
        currentUserId = null
    }
}
