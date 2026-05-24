import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:nomi_mobile/core/api/mqtt_service.dart';

class MqttLifecycleNotifier extends Notifier<void> {
  @override
  void build() {
    final authState = ref.watch(authProvider);
    final mqtt = ref.read(mqttServiceProvider);

    // 🛑 CRITICAL FIX: Do nothing while the auth state is still loading/checking
    if (authState.isLoading) return;

    if (authState.isAuthenticated && authState.user != null) {
      print('[MQTT] Definitive session found. Ensuring global connection...');
      _startConnection(mqtt, authState.user!.id);
    } else {
      print('[MQTT] No active session. Ensuring global disconnection...');
      mqtt.disconnect();
    }
  }

  Future<void> _startConnection(MqttService mqtt, String userId) async {
    await mqtt.connect(userId);
  }
}

final mqttLifecycleProvider = NotifierProvider<MqttLifecycleNotifier, void>(() {
  return MqttLifecycleNotifier();
});
