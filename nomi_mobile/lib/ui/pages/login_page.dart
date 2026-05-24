import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';

class LoginPage extends ConsumerStatefulWidget {
  const LoginPage({super.key});

  @override
  ConsumerState<LoginPage> createState() => _LoginPageState();
}

class _LoginPageState extends ConsumerState<LoginPage> {
  final _idController = TextEditingController();
  final _otpController = TextEditingController();
  String _selectedChannel = 'whatsapp';

  @override
  Widget build(BuildContext context) {
    final authState = ref.watch(authProvider);

    final List<Map<String, dynamic>> channels = [
      {'id': 'whatsapp', 'label': 'WhatsApp', 'icon': LucideIcons.messageCircle},
      {'id': 'telegram', 'label': 'Telegram', 'icon': LucideIcons.send},
      {'id': 'email', 'label': 'Email', 'icon': LucideIcons.mail},
    ];

    return Scaffold(
      backgroundColor: const Color(0xFF020617),
      body: Stack(
        children: [
          Center(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(32.0),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  // Logo/Icon
                  Container(
                    padding: const EdgeInsets.all(24),
                    decoration: BoxDecoration(
                      color: Colors.blue.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(32),
                      border: Border.all(color: Colors.blue.withValues(alpha: 0.2)),
                    ),
                    child: const Icon(LucideIcons.bot, size: 64, color: Colors.blue),
                  ),
                  const SizedBox(height: 32),
                  const Text(
                    'Nomi AI',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: 32,
                      fontWeight: FontWeight.w900,
                      letterSpacing: -1,
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'High-Fidelity Agentic Workspace',
                    style: TextStyle(
                      color: Colors.white.withValues(alpha: 0.5),
                      fontSize: 14,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(height: 48),

                  // Inputs
                  if (!authState.otpSent) ...[
                    _buildChannelSelector(channels),
                    const SizedBox(height: 24),
                    _buildInput(
                      controller: _idController,
                      label: 'IDENTITY ID',
                      hint: _selectedChannel == 'email' ? 'Enter your email' : 'Enter phone or ID',
                      icon: LucideIcons.user,
                    ),
                    const SizedBox(height: 24),
                    _buildButton(
                      onPressed: () async {
                        if (_idController.text.isNotEmpty) {
                          await ref
                              .read(authProvider.notifier)
                              .requestOtp(_idController.text, _selectedChannel);
                        }
                      },
                      label: 'REQUEST ACCESS',
                      isLoading: authState.isLoading,
                    ),
                  ] else ...[
                    _buildInput(
                      controller: _otpController,
                      label: 'OTP VERIFICATION',
                      hint: 'Enter 6-digit code',
                      icon: LucideIcons.shieldCheck,
                      keyboardType: TextInputType.number,
                    ),
                    const SizedBox(height: 24),
                    _buildButton(
                      onPressed: () async {
                        if (_otpController.text.isNotEmpty) {
                          await ref
                              .read(authProvider.notifier)
                              .verifyOtp(authState.identity ?? _idController.text, _otpController.text);
                        }
                      },
                      label: 'VERIFY & ENTER',
                      isLoading: authState.isLoading,
                    ),
                    TextButton(
                      onPressed: () => ref.read(authProvider.notifier).resetOtp(),
                      child: const Text('Change ID', style: TextStyle(color: Colors.grey)),
                    ),
                  ],
                ],
              ),
            ),
          ),
          
          if (authState.isLoading)
            Container(
              color: Colors.black.withValues(alpha: 0.5),
              child: const Center(
                child: CircularProgressIndicator(color: Colors.blue),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildChannelSelector(List<Map<String, dynamic>> channels) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Padding(
          padding: EdgeInsets.only(left: 4, bottom: 12),
          child: Text(
            'SELECT CHANNEL',
            style: TextStyle(
              color: Colors.blue,
              fontSize: 10,
              fontWeight: FontWeight.w900,
              letterSpacing: 1.5,
            ),
          ),
        ),
        Row(
          children: channels.map((ch) {
            final isActive = _selectedChannel == ch['id'];
            return Expanded(
              child: GestureDetector(
                onTap: () => setState(() => _selectedChannel = ch['id']),
                child: Container(
                  margin: EdgeInsets.only(
                    right: ch['id'] != channels.last['id'] ? 8 : 0,
                  ),
                  padding: const EdgeInsets.symmetric(vertical: 12),
                  decoration: BoxDecoration(
                    color: isActive ? Colors.blue.withValues(alpha: 0.1) : Colors.white.withValues(alpha: 0.05),
                    borderRadius: BorderRadius.circular(16),
                    border: Border.all(
                      color: isActive ? Colors.blue : Colors.white.withValues(alpha: 0.1),
                    ),
                  ),
                  child: Column(
                    children: [
                      Icon(
                        ch['icon'],
                        size: 18,
                        color: isActive ? Colors.blue : Colors.white.withValues(alpha: 0.5),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        ch['label'],
                        style: TextStyle(
                          color: isActive ? Colors.white : Colors.white.withValues(alpha: 0.5),
                          fontSize: 9,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            );
          }).toList(),
        ),
      ],
    );
  }

  Widget _buildInput({
    required TextEditingController controller,
    required String label,
    required String hint,
    required IconData icon,
    TextInputType? keyboardType,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.only(left: 4, bottom: 8),
          child: Text(
            label,
            style: const TextStyle(
              color: Colors.blue,
              fontSize: 10,
              fontWeight: FontWeight.w900,
              letterSpacing: 1.5,
            ),
          ),
        ),
        Container(
          decoration: BoxDecoration(
            color: Colors.white.withValues(alpha: 0.05),
            borderRadius: BorderRadius.circular(16),
            border: Border.all(color: Colors.white.withValues(alpha: 0.1)),
          ),
          child: TextField(
            controller: controller,
            keyboardType: keyboardType,
            style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold),
            decoration: InputDecoration(
              hintText: hint,
              hintStyle: TextStyle(color: Colors.white.withValues(alpha: 0.2)),
              prefixIcon: Icon(icon, size: 18, color: Colors.white.withValues(alpha: 0.5)),
              border: InputBorder.none,
              contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildButton({
    required VoidCallback onPressed,
    required String label,
    bool isLoading = false,
  }) {
    return SizedBox(
      width: double.infinity,
      height: 56,
      child: ElevatedButton(
        onPressed: isLoading ? null : onPressed,
        style: ElevatedButton.styleFrom(
          backgroundColor: Colors.blue,
          foregroundColor: Colors.white,
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
          elevation: 0,
        ),
        child: isLoading
            ? const SizedBox(
                width: 20,
                height: 20,
                child: CircularProgressIndicator(color: Colors.white, strokeWidth: 2),
              )
            : Text(
                label,
                style: const TextStyle(
                  fontWeight: FontWeight.w900,
                  letterSpacing: 1,
                  fontSize: 12,
                ),
              ),
      ),
    );
  }
}
