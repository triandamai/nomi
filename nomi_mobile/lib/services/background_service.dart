import 'dart:async';
import 'dart:ui';
import 'package:flutter_background_service/flutter_background_service.dart';
import 'package:mqtt_client/mqtt_client.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:nomi_mobile/core/api/mqtt_service.dart';

@pragma('vm:entry-point')
Future<void> initializeBackgroundService() async {
  final service = FlutterBackgroundService();

  await service.configure(
    androidConfiguration: AndroidConfiguration(
      onStart: onStart,
      autoStart: true,
      isForegroundMode: true,
      notificationChannelId: 'nomi_background',
      initialNotificationTitle: 'Nomi AI',
      initialNotificationContent: 'Agentic workspace active...',
    ),
    iosConfiguration: IosConfiguration(
      autoStart: true,
      onForeground: onStart,
      onBackground: onIosBackground,
    ),
  );
}

@pragma('vm:entry-point')
Future<bool> onIosBackground(ServiceInstance service) async {
  return true;
}
@pragma('vm:entry-point')
void onStart(ServiceInstance service) async {
  runZonedGuarded(() async {
    print('[Background] 🌀 Isolate Start Sequence Initiated');
    DartPluginRegistrant.ensureInitialized();

    const storage = FlutterSecureStorage();
    MqttService? mqtt;

    service.on('connect').listen((event) async {
      print('[Background] 📡 Connection signal received');
      final uid = await storage.read(key: 'user_id');
      if (uid != null) {
        mqtt ??= MqttService();
        _initMqtt(mqtt!, service, uid);
      } else {
        print('[Background] ⚠️ Cannot connect: No User ID in storage');
      }
    });

    service.on('disconnect').listen((event) {
      print('[Background] 🔌 Disconnect signal received');
      mqtt?.disconnect();
    });

    service.on('stopService').listen((event) {
      print('[Background] 🛑 Stopping service');
      mqtt?.disconnect();
      service.stopSelf();
    });

    // Auto-resume check
    final token = await storage.read(key: 'jwt_token');
    final userId = await storage.read(key: 'user_id');
    if (token != null && userId != null) {
      print('[Background] 🔄 Session found, auto-resuming MQTT...');
      mqtt ??= MqttService();
      _initMqtt(mqtt!, service, userId);
    }

    print('[Background] ✅ Initialization complete. Listening for events.');
  }, (error, stack) {
    print('[Background] ❌ CRITICAL ISOLATE ERROR: $error');
    print(stack);
  });
}

Future<void> _initMqtt(MqttService mqtt, ServiceInstance service, String userId) async {
  if (mqtt.isConnected) return;

  final status = await mqtt.connect(userId);
  if (status?.state == MqttConnectionState.connected) {
    mqtt.updates?.listen((msgList) {
      if (msgList.isEmpty) return;
      final MqttPublishMessage rec = msgList[0].payload as MqttPublishMessage;
      final String payload = MqttPublishPayload.bytesToStringAsString(rec.payload.message);
      
      service.invoke('onMqttMessage', {
        'topic': msgList[0].topic,
        'payload': payload,
      });
    });
  }
}
