import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mqtt_client/mqtt_client.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:nomi_mobile/providers/chat_provider.dart';
import 'package:nomi_mobile/core/api/mqtt_service.dart';

class MqttLifecycleNotifier extends Notifier<void> {
  @override
  void build() {
    final authState = ref.watch(authProvider);
    final mqtt = ref.read(mqttServiceProvider);
    final conversationsAsync = ref.watch(conversationsStreamProvider);

    // 🛑 CRITICAL FIX: Do nothing while the auth state is still loading/checking
    if (authState.isLoading) return;

    if (authState.isAuthenticated && authState.user != null) {
      final conversations = conversationsAsync.asData?.value ?? [];
      
      if (mqtt.isConnected) {
        // dynamically subscribe to any active conversation topics
        for (final conv in conversations) {
          mqtt.subscribe('nomi/conversations/${conv.id}/#');
        }
      } else {
        print('[MQTT] Definitive session found. Ensuring global connection...');
        _startConnection(mqtt, authState.user!.id, conversations);
      }
    } else {
      print('[MQTT] No active session. Ensuring global disconnection...');
      mqtt.disconnect();
    }
  }

  Future<void> _startConnection(MqttService mqtt, String userId, List<dynamic> conversations) async {
    final status = await mqtt.connect(userId);
    if (status?.state == MqttConnectionState.connected) {
      for (final conv in conversations) {
        mqtt.subscribe('nomi/conversations/${conv.id}/#');
      }
    }
  }
}

final mqttLifecycleProvider = NotifierProvider<MqttLifecycleNotifier, void>(() {
  return MqttLifecycleNotifier();
});
