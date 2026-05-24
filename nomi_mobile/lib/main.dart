import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:nomi_mobile/providers/mqtt_provider.dart';
import 'package:nomi_mobile/ui/pages/login_page.dart';
import 'package:nomi_mobile/ui/pages/chat_page.dart';

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

    return MaterialApp(
      title: 'Nomi AI',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        brightness: Brightness.dark,
        primarySwatch: Colors.blue,
        scaffoldBackgroundColor: const Color(0xFF020617),
        textTheme: GoogleFonts.interTextTheme(
          ThemeData.dark().textTheme,
        ),
      ),
      home: authState.isAuthenticated
              ? const ChatPage()
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
