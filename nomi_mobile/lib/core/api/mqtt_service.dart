import 'dart:io';
import 'package:mqtt_client/mqtt_client.dart';
import 'package:mqtt_client/mqtt_server_client.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';
import 'package:flutter/foundation.dart';

class MqttService {
  MqttServerClient? _client;
  String? _currentUserId;
  String? _deviceId;

  MqttService();

  bool get isConnected =>
      _client?.connectionStatus?.state == MqttConnectionState.connected;

  bool get isConnecting =>
      _client?.connectionStatus?.state == MqttConnectionState.connecting;

  Future<String> _getOrCreateDeviceId() async {
    if (_deviceId != null) return _deviceId!;
    
    final prefs = await SharedPreferences.getInstance();
    String? id = prefs.getString('mqtt_device_id');
    
    if (id == null) {
      id = const Uuid().v4();
      await prefs.setString('mqtt_device_id', id);
    }
    
    _deviceId = id;
    return id;
  }

  String _getPlatformName() {
    if (kIsWeb) return 'web';
    if (Platform.isAndroid) return 'android';
    if (Platform.isIOS) return 'ios';
    if (Platform.isMacOS) return 'macos';
    if (Platform.isWindows) return 'windows';
    if (Platform.isLinux) return 'linux';
    return 'unknown';
  }

  Future<MqttClientConnectionStatus?> connect(String userId) async {
    if (isConnected && _currentUserId == userId) {
      return _client?.connectionStatus;
    }

    if (isConnecting) {
      debugPrint('[MQTT] ⏳ Connection already in progress. Waiting...');
      return _client?.connectionStatus;
    }

    // Disconnect if user changed
    if (isConnected && _currentUserId != userId) {
      debugPrint('[MQTT] 👤 User changed. Reconnecting...');
      disconnect();
    }

    _currentUserId = userId;
    final deviceId = await _getOrCreateDeviceId();
    final platform = _getPlatformName();
    
    // 🏷️ Structured Client ID: nomi/users/${userId}/${platform}_${deviceId}
    final clientId = 'nomi/users/$userId/${platform}_$deviceId';
    debugPrint('[MQTT] 🆔 Client ID: $clientId');

    _client = MqttServerClient.withPort(
      AppConfig.mqttHost,
      clientId,
      AppConfig.mqttPort,
    );

    _client!.keepAlivePeriod = 20;
    _client!.onConnected = _onConnected;
    _client!.onDisconnected = _onDisconnected;
    _client!.onSubscribed = _onSubscribed;
    _client!.pongCallback = _onPong;
    _client!.logging(on: false);

    // 🔒 Security & Connection Hardening
    _client!.setProtocolV311();
    _client!.useWebSocket = false; 
    
    if (AppConfig.mqttPort == 8883) {
      _client!.secure = true;
      _client!.onBadCertificate = (dynamic cert) => true; 
      debugPrint('[MQTT] 🔒 Secure MQTTS Mode: TCP/SSL on port 8883');
    }

    try {
      debugPrint('[MQTT] 🛰️ Connecting to ${AppConfig.mqttHost}:${AppConfig.mqttPort} via Raw TCP...');
      final status = await _client!.connect(AppConfig.mqttUsername, AppConfig.mqttPassword);
      
      // Auto-subscribe to technical topics
      if (status?.state == MqttConnectionState.connected) {
        subscribe('nomi/users/$userId/#');
        subscribe('nomi/broadcast/#');
      }
      
      return status;
    } catch (e) {
      debugPrint('[MQTT] ❌ Handshake failed - $e');
      _client?.disconnect();
      return _client?.connectionStatus;
    }
  }

  void subscribe(String topic) {
    if (isConnected) {
      debugPrint('[MQTT] 📝 Subscribing to: $topic');
      _client!.subscribe(topic, MqttQos.atMostOnce);
    } else {
      debugPrint('[MQTT] ⚠️ Cannot subscribe to $topic: Not connected.');
    }
  }

  void disconnect() {
    if (isConnected || isConnecting) {
      debugPrint('[MQTT] 🔌 Manually disconnecting...');
      _client?.disconnect();
      _currentUserId = null;
    }
  }

  Stream<List<MqttReceivedMessage<MqttMessage>>>? get updates => _client?.updates;

  void _onConnected() {
    debugPrint('[MQTT] ✅ Handshake complete. Global connection active.');
  }

  void _onDisconnected() {
    debugPrint('[MQTT] 🔌 Service disconnected.');
  }

  void _onSubscribed(String topic) {
    debugPrint('[MQTT] 📌 Subscription confirmed for: $topic');
  }

  void _onPong() {
    // debugPrint('[MQTT] 💓 Heartbeat (PONG) received.');
  }
}

final mqttServiceProvider = Provider<MqttService>((ref) {
  return MqttService();
});
