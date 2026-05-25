import mqtt from 'mqtt';
import {eventBus} from '$lib/utils';
import {getSession} from '$lib/stores/profile.svelte';
import {env} from '$env/dynamic/public';

const MQTT_URL = env.PUBLIC_MQTT_URL || 'wss://broker.pakaiarta.id/mqtt';

class MqttClient {
    private client: mqtt.MqttClient | null = null;
    private userId: string | null = null;
    private currentConversationId: string | null = null;

    connect() {
        if (this.client) return;

        const [token, userId] = getSession();
        if (!userId) {
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

        const existingDeviceID = sessionStorage.getItem("13caZza")
        const candidateDeviceId = crypto.randomUUID().slice(0, 8);

        const deviceId = ()=>{
            if (existingDeviceID){
                return existingDeviceID
            }
            sessionStorage.setItem("13caZza",candidateDeviceId)
            return candidateDeviceId
        }
        // const deviceId = crypto.randomUUID().slice(0, 8);
        // Using a hierarchical client ID allows EMQX ACLs to use %c wildcards easily
        const clientId = `nomi/users/${this.userId}/web_${deviceId()}`;


        this.client = mqtt.connect(finalUrl, {
            clientId: clientId, 
            clean: false,       
            username: env.PUBLIC_MQTT_USER,
            password: env.PUBLIC_MQTT_PASSWORD,
            connectTimeout: 10000,
            reconnectPeriod: 2000, 
            protocolVersion: 4,
        });

        this.client.on('connect', () => {
            eventBus.emit('gateway-status', {online: true, transport: 'mqtt'});
            this.subscribeToBasicTopics();
        });

        this.client.on('disconnect', () => {
            // eventBus.emit('gateway-status', {online: false, transport: 'mqtt'});
        })
        this.client.on('reconnect', () => {
            // console.log('MQTT: Reconnected');
        })
        this.client.on('offline', () => {
            // console.log('MQTT: Offline');
        })
        this.client.on('error', (err) => {
            eventBus.emit('gateway-status', {online: false, transport: 'mqtt'});
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
                // console.error('MQTT: Subscription error', err);
            } else {
                // console.log('MQTT: Subscribed to basic topics');
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
                    // console.error('MQTT: Conversation subscription error', err);
                } else {
                    // console.log(`MQTT: Subscribed to conversation ${this.currentConversationId}`);
                }
            });
        }
    }

    private handleMessage(topic: string, message: string) {
        try {
            const data = JSON.parse(message);
            const parts = topic.split('/');
            const eventName = parts[parts.length - 1];


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
                case 'deb_update':
                    eventBus.emit('sse-deb_update', data);
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
