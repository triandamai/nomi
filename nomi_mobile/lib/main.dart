import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:nomi_mobile/providers/mqtt_provider.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/ui/pages/login_page.dart';
import 'package:nomi_mobile/ui/pages/splash_screen.dart';
import 'package:nomi_mobile/ui/widgets/main_shell.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const ProviderScope(child: NomiApp()));
}

class NomiApp extends ConsumerWidget {
  const NomiApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final authState = ref.watch(authProvider);
    ref.watch(mqttLifecycleProvider); // Global MQTT Lifecycle
    final themeState = ref.watch(themeProvider);

    return MaterialApp(
      title: 'Nomi AI',
      debugShowCheckedModeBanner: false,
      theme: themeState.toThemeData(),
      home: !authState.isInitialChecked
          ? const NomiSplashScreen()
          : authState.isAuthenticated
              ? const MainShell()
              : const LoginPage(),
    );
  }
}

class LoadingScreen extends StatelessWidget {
  const LoadingScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(
        child: CircularProgressIndicator(color: Colors.blue),
      ),
    );
  }
}
