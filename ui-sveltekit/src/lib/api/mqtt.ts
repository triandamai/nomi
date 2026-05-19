import mqtt from 'mqtt';
import { eventBus } from '$lib/utils';
import { getSession } from '$lib/stores/profile.svelte';
import { env } from '$env/dynamic/public';

const MQTT_URL = env.PUBLIC_MQTT_URL || 'wss://broker.pakaiarta.id';

class MqttClient {
    private client: mqtt.MqttClient | null = null;
    private userId: string | null = null;
    private currentConversationId: string | null = null;

    connect() {
        if (this.client) return;

        const [token, userId] = getSession();
        if (!userId) {
            console.warn('MQTT: No user ID found, skipping connection');
            return;
        }

        this.userId = userId;
        // For EMQX Cloud WSS, port 8084 is the standard.
        let finalUrl = MQTT_URL;
        if (finalUrl.includes('emqxsl.com')) {
            const urlObj = new URL(finalUrl);
            if (!urlObj.port) {
                urlObj.port = '8084';
                finalUrl = urlObj.toString();
            }
        }

        console.log('MQTT: Connecting to', finalUrl);

        this.client = mqtt.connect(finalUrl, {
            clientId: this.userId, // Use the real User UUID for ACL mapping
            clean: true,
            username: env.PUBLIC_MQTT_USER,
            password:  env.PUBLIC_MQTT_PASSWORD,
            connectTimeout: 10000,
            reconnectPeriod: 2000,
            protocolVersion: 4,
        });

        this.client.on('connect', () => {
            console.log('MQTT: Connected');
            eventBus.emit('gateway-status', { online: true, transport: 'mqtt' });
            this.subscribeToBasicTopics();
        });

        this.client.on('error', (err) => {
            console.error('MQTT: Connection error', err);
            eventBus.emit('gateway-status', { online: false, transport: 'mqtt' });
        });

        this.client.on('message', (topic, payload) => {
            this.handleMessage(topic, payload.toString());
        });
    }

    private subscribeToBasicTopics() {
        if (!this.client || !this.userId) return;

        const topics = [
            `nomi/users/${this.userId}/#`,
            `nomi/broadcast/#`
        ];

        this.client.subscribe(topics, (err) => {
            if (err) {
                console.error('MQTT: Subscription error', err);
            } else {
                console.log('MQTT: Subscribed to basic topics');
            }
        });
    }

    setConversation(conversationId: string | null) {
        if (!this.client) return;

        // Unsubscribe from previous conversation
        if (this.currentConversationId && this.currentConversationId !== conversationId) {
            this.client.unsubscribe(`nomi/conversations/${this.currentConversationId}/#`);
        }

        this.currentConversationId = conversationId;

        if (this.currentConversationId) {
            this.client.subscribe(`nomi/conversations/${this.currentConversationId}/#`, (err) => {
                if (err) {
                    console.error('MQTT: Conversation subscription error', err);
                } else {
                    console.log(`MQTT: Subscribed to conversation ${this.currentConversationId}`);
                }
            });
        }
    }

    private handleMessage(topic: string, message: string) {
        try {
            const data = JSON.parse(message);
            const parts = topic.split('/');
            const eventName = parts[parts.length - 1];

            console.log(`MQTT: Received [${eventName}] on [${topic}]`);

            // Map MQTT topic events back to SSE-style eventBus events for legacy compatibility
            switch (eventName) {
                case 'message':
                    eventBus.emit('sse-message', data);
                    break;
                case 'metadata':
                    eventBus.emit('sse-metadata', data);
                    break;
                case 'thought':
                    eventBus.emit('sse-thought', data);
                    break;
                case 'tool_start':
                    eventBus.emit('sse-tool_start', data);
                    break;
                case 'tool_end':
                    eventBus.emit('sse-tool_end', data);
                    break;
                case 'token_update':
                    eventBus.emit('sse-token_update', data);
                    break;
                case 'presence':
                    eventBus.emit('sse-presence', data);
                    break;
                case 'pairing-success':
                    eventBus.emit('sse-pairing-success', data);
                    break;
                case 'evolution':
                    eventBus.emit('sse-evolution', data);
                    break;
                default:
                    // Generic handler for unknown events
                    eventBus.emit(`mqtt-${eventName}`, data);
            }
        } catch (e) {
            console.error('MQTT: Failed to parse message', e);
        }
    }

    disconnect() {
        if (this.client) {
            this.client.end();
            this.client = null;
        }
    }
}

export const mqttClient = new MqttClient();
