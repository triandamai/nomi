import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';
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
    final themeState = ref.watch(themeProvider);
    final authState = ref.watch(authProvider);

    final List<Map<String, dynamic>> channels = [
      {'id': 'whatsapp', 'label': 'WhatsApp', 'icon': LucideIcons.messageCircle},
      {'id': 'telegram', 'label': 'Telegram', 'icon': LucideIcons.send},
      {'id': 'email', 'label': 'Email', 'icon': LucideIcons.mail},
    ];

    return Scaffold(
      backgroundColor: themeState.isDark ? const Color(0xFF020617) : Color(themeState.bgHeader),
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
                      color: Color(themeState.primaryColor).withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(32),
                      border: Border.all(color: Color(themeState.primaryColor).withValues(alpha: 0.2)),
                    ),
                    child: Icon(LucideIcons.bot, size: 64, color: Color(themeState.primaryColor)),
                  ),
                  const SizedBox(height: 32),
                  Text(
                    'Nomi AI',
                    style: TextStyle(
                      color: Color(themeState.textMain),
                      fontSize: 32,
                      fontWeight: FontWeight.w900,
                      letterSpacing: -1,
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'High-Fidelity Agentic Workspace',
                    style: TextStyle(
                      color: Color(themeState.textMuted),
                      fontSize: 14,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(height: 48),

                  // Inputs
                  if (!authState.otpSent) ...[
                    _buildChannelSelector(themeState, channels),
                    const SizedBox(height: 24),
                    _buildInput(
                      themeState: themeState,
                      controller: _idController,
                      label: 'IDENTITY ID',
                      hint: _selectedChannel == 'email' ? 'Enter your email' : 'Enter phone or ID',
                      icon: LucideIcons.user,
                    ),
                    const SizedBox(height: 24),
                    _buildButton(
                      themeState: themeState,
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
                      themeState: themeState,
                      controller: _otpController,
                      label: 'OTP VERIFICATION',
                      hint: 'Enter 6-digit code',
                      icon: LucideIcons.shieldCheck,
                      keyboardType: TextInputType.number,
                    ),
                    const SizedBox(height: 24),
                    _buildButton(
                      themeState: themeState,
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
                      child: Text('Change ID', style: TextStyle(color: Color(themeState.textMuted))),
                    ),
                  ],
                ],
              ),
            ),
          ),
          
          if (authState.isLoading)
            Container(
              color: Colors.black.withValues(alpha: 0.5),
              child: Center(
                child: CircularProgressIndicator(color: Color(themeState.primaryColor)),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildChannelSelector(NomiTheme themeState, List<Map<String, dynamic>> channels) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.only(left: 4, bottom: 12),
          child: Text(
            'SELECT CHANNEL',
            style: TextStyle(
              color: Color(themeState.primaryColor),
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
                    color: isActive ? Color(themeState.primaryColor).withValues(alpha: 0.1) : Color(themeState.textMain).withValues(alpha: 0.03),
                    borderRadius: BorderRadius.circular(16),
                    border: Border.all(
                      color: isActive ? Color(themeState.primaryColor) : Color(themeState.borderMain).withValues(alpha: 0.5),
                    ),
                  ),
                  child: Column(
                    children: [
                      Icon(
                        ch['icon'],
                        size: 18,
                        color: isActive ? Color(themeState.primaryColor) : Color(themeState.textMuted),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        ch['label'],
                        style: TextStyle(
                          color: isActive ? Color(themeState.textMain) : Color(themeState.textMuted),
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
    required NomiTheme themeState,
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
            style: TextStyle(
              color: Color(themeState.primaryColor),
              fontSize: 10,
              fontWeight: FontWeight.w900,
              letterSpacing: 1.5,
            ),
          ),
        ),
        Container(
          decoration: BoxDecoration(
            color: themeState.isDark ? Colors.white.withValues(alpha: 0.05) : Colors.black.withValues(alpha: 0.06),
            borderRadius: BorderRadius.circular(16),
            border: Border.all(color: Color(themeState.borderMain).withValues(alpha: 0.5)),
          ),
          child: TextField(
            controller: controller,
            keyboardType: keyboardType,
            style: TextStyle(color: Color(themeState.textMain), fontWeight: FontWeight.bold),
            decoration: InputDecoration(
              hintText: hint,
              hintStyle: TextStyle(color: Color(themeState.textMuted).withValues(alpha: 0.5)),
              prefixIcon: Icon(icon, size: 18, color: Color(themeState.textMuted)),
              border: InputBorder.none,
              contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildButton({
    required NomiTheme themeState,
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
          backgroundColor: Color(themeState.primaryColor),
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
