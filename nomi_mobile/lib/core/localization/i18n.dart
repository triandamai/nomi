import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_riverpod/legacy.dart';

final localeProvider = StateProvider<String>((ref) => 'en');

class I18n {
  static final Map<String, Map<String, String>> _translations = {
    'en': {
      // Splash & Core
      'nomi': 'NOMI',
      'autonomous_orchestration_os': 'Autonomous Orchestration OS',
      'loading': 'Loading...',
      'message_nomi': 'Message Nomi...',
      'replying_to': 'Replying to',
      'sync_error': 'Sync Error',
      'private_sandbox': 'Private Sandbox',
      
      // Sidebar & Navigation
      'choose_conversation': 'Choose a conversation from the sidebar to begin technical operations.',
      'admin_panel': 'Admin Panel',
      'autonomous_workflows': 'Autonomous Workflows',
      'health_history': 'Health History',
      'reminders': 'Reminders',
      'transactions': 'Transactions',
      'plugins': 'Plugins',
      'skills': 'Skills',
      'storage': 'Storage',
      'guardrails': 'Guardrails',
      'reinforcement': 'Reinforcement',

      // Settings
      'profile_settings': 'Profile Settings',
      'display_name': 'Display Name',
      'select_language': 'Language',
      'save': 'Save',
      'cancel': 'Cancel',
      'logout': 'Logout',
      'user': 'User',
      'admin': 'Admin',

      // Chat & Process
      'active_process': 'Active Process',
      'thinking': 'Thinking...',
      'tool_execution': 'Tool Execution',

      // Details Sheet
      'conversation_parameters': 'Conversation Parameters',
      'interaction_gate': 'Interaction Gate',
      'intent_classification': 'Intent Classification',
      'guardrail_level': 'Guardrail Level',
      'close': 'Close',

      // Interaction Modes
      'proactive': 'Proactive',
      'balanced': 'Balanced',
      'conservative': 'Conservative',
      'silent_monitor': 'Silent Monitor',

      // Intent Modes
      'experimental': 'Experimental',
      'adaptive': 'Adaptive',
      'strict': 'Strict',

      // Guardrail Modes
      'permissive': 'Permissive',
      'standard': 'Standard',
      'hardened_shield': 'Hardened Shield',
    },
    'id': {
      // Splash & Core
      'nomi': 'NOMI',
      'autonomous_orchestration_os': 'Sistem Operasi Orkestrasi Otonom',
      'loading': 'Memuat...',
      'message_nomi': 'Kirim pesan ke Nomi...',
      'replying_to': 'Membalas ke',
      'sync_error': 'Kesalahan Sinkronisasi',
      'private_sandbox': 'Sandbox Pribadi',
      
      // Sidebar & Navigation
      'choose_conversation': 'Pilih percakapan dari bilah samping untuk memulai operasi teknis.',
      'admin_panel': 'Panel Admin',
      'autonomous_workflows': 'Alur Kerja Otonom',
      'health_history': 'Riwayat Kesehatan',
      'reminders': 'Pengingat',
      'transactions': 'Transaksi',
      'plugins': 'Plugin',
      'skills': 'Keahlian',
      'storage': 'Penyimpanan',
      'guardrails': 'Pagar Pengaman',
      'reinforcement': 'Penguatan',

      // Settings
      'profile_settings': 'Pengaturan Profil',
      'display_name': 'Nama Tampilan',
      'select_language': 'Bahasa',
      'save': 'Simpan',
      'cancel': 'Batal',
      'logout': 'Keluar',
      'user': 'Pengguna',
      'admin': 'Admin',

      // Chat & Process
      'active_process': 'Proses Aktif',
      'thinking': 'Berpikir...',
      'tool_execution': 'Eksekusi Alat',

      // Details Sheet
      'conversation_parameters': 'Parameter Percakapan',
      'interaction_gate': 'Gerbang Interaksi',
      'intent_classification': 'Klasifikasi Niat',
      'guardrail_level': 'Tingkat Pagar Pengaman',
      'close': 'Tutup',

      // Interaction Modes
      'proactive': 'Proaktif',
      'balanced': 'Seimbang',
      'conservative': 'Konservatif',
      'silent_monitor': 'Monitor Diam',

      // Intent Modes
      'experimental': 'Eksperimental',
      'adaptive': 'Adaptif',
      'strict': 'Ketat',

      // Guardrail Modes
      'permissive': 'Permisif',
      'standard': 'Standar',
      'hardened_shield': 'Perisai Kokoh',
    }
  };

  static String get(String key, String locale) {
    return _translations[locale]?[key] ?? _translations['en']?[key] ?? key;
  }
}

extension Trans on String {
  String tr(WidgetRef ref) {
    final locale = ref.watch(localeProvider);
    return I18n.get(this, locale);
  }
}
