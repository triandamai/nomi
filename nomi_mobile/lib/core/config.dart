class AppConfig {
  static const String baseUrl = String.fromEnvironment(
    'GATEWAY_BASE_URL',
    defaultValue: 'http://localhost:8000/api',
  );
  static const String fileUrl = String.fromEnvironment(
    'GATEWAY_FILE_URL',
    defaultValue: 'http://localhost:8000/api/files',
  );
  static const String mqttHost = String.fromEnvironment(
    'MQTT_HOST',
    defaultValue: 'b1fec516.ala.eu-central-1.emqxsl.com',
  );
  static const int mqttPort = int.fromEnvironment(
    'MQTT_PORT',
    defaultValue: 8883,
  );
  static const String mqttUsername = String.fromEnvironment(
    'MQTT_USERNAME',
    defaultValue: 'nomi-client-app',
  );
  static const String mqttPassword = String.fromEnvironment(
    'MQTT_PASSWORD',
    defaultValue: 'NomiPublicPass2026',
  );

  
  // Nomi Visual Palette
  static const int deepSlate = 0xFF020617;
  static const int sidebarBg = 0xFF111b21;
  static const int emerald = 0xFF10b981;
  static const int indigo = 0xFF6366f1;
  static const int blue = 0xFF3b82f6;
  static const int amber = 0xFFf59e0b;
  static const int rose = 0xFFef4444;
}
