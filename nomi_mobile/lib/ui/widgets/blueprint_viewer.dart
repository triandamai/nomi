import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:webview_flutter/webview_flutter.dart';
import 'package:lucide_icons_flutter/lucide_icons.dart';
import 'package:nomi_mobile/core/config.dart';
import 'package:nomi_mobile/providers/theme_provider.dart';
import 'package:nomi_mobile/providers/auth_provider.dart';
import 'dart:ui';

class BlueprintViewerSheet extends ConsumerStatefulWidget {
  const BlueprintViewerSheet({super.key});

  @override
  ConsumerState<BlueprintViewerSheet> createState() => _BlueprintViewerSheetState();
}

class _BlueprintViewerSheetState extends ConsumerState<BlueprintViewerSheet> {
  late final WebViewController _controller;
  bool _isLoading = true;
  double _progress = 0;

  @override
  void initState() {
    super.initState();
    
    // 🌐 Extract SvelteKit Base URL (Assume same host as API)
    final baseUrl = AppConfig.baseUrl.replaceFirst('/api', '');
    
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..setBackgroundColor(const Color(AppConfig.deepSlate))
      ..setNavigationDelegate(
        NavigationDelegate(
          onProgress: (int progress) {
            if (mounted) setState(() => _progress = progress / 100);
          },
          onPageStarted: (String url) {
            if (mounted) setState(() => _isLoading = true);
          },
          onPageFinished: (String url) async {
            if (mounted) setState(() => _isLoading = false);
            
            // 🔐 Inject Authentication Token into WebView's LocalStorage
            final auth = ref.read(authProvider);
            if (auth.token != null) {
              await _controller.runJavaScript('''
                localStorage.setItem('jwt_token', '${auth.token}');
                console.log('Nomi: Auth token injected successfully.');
              ''');
              // Reload to ensure the app picks up the token if needed, 
              // or just let the app handle it if it checks localStorage on mount.
              // For SvelteKit RAG page, it usually fetches data on mount.
            }
          },
        ),
      )
      ..loadRequest(Uri.parse('$baseUrl/rag'));
  }

  @override
  Widget build(BuildContext context) {
    final themeState = ref.watch(themeProvider);
    final size = MediaQuery.of(context).size;

    return ClipRRect(
      borderRadius: const BorderRadius.only(
        topLeft: Radius.circular(20),
        topRight: Radius.circular(20),
      ),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 12, sigmaY: 12),
        child: Container(
          width: double.infinity,
          constraints: BoxConstraints(maxHeight: size.height * 0.9),
          decoration: BoxDecoration(
            color: themeState.isDark 
              ? Color(themeState.slate950).withValues(alpha: 0.85) 
              : Color(themeState.bgHeader).withValues(alpha: 0.92),
            border: Border.all(
              color: Color(themeState.borderMain).withValues(alpha: 0.25),
              width: 1.2,
            ),
            borderRadius: const BorderRadius.only(
              topLeft: Radius.circular(20),
              topRight: Radius.circular(20),
            ),
          ),
          child: Column(
        children: [
          // Header with Liquid Glass Feel
          ClipRRect(
            child: BackdropFilter(
              filter: ImageFilter.blur(sigmaX: 10, sigmaY: 10),
              child: Container(
                padding: const EdgeInsets.all(24),
                decoration: BoxDecoration(
                  color: Colors.white.withValues(alpha: 0.02),
                  border: Border(bottom: BorderSide(color: Colors.white.withValues(alpha: 0.05))),
                ),
                child: Column(
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        const Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'KNOWLEDGE GRAPH',
                              style: TextStyle(
                                color: Color(AppConfig.emerald),
                                fontSize: 10,
                                fontWeight: FontWeight.w900,
                                letterSpacing: 2,
                              ),
                            ),
                            SizedBox(height: 4),
                            Text(
                              'System Blueprint',
                              style: TextStyle(color: Colors.white, fontSize: 22, fontWeight: FontWeight.bold),
                            ),
                          ],
                        ),
                        Row(
                          children: [
                            IconButton(
                              onPressed: () => _controller.reload(),
                              icon: const Icon(LucideIcons.refreshCw, color: Colors.white38, size: 18),
                            ),
                            IconButton(
                              onPressed: () => Navigator.pop(context),
                              icon: const Icon(LucideIcons.x, color: Colors.white38),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ),

          // Progress Bar
          if (_isLoading)
            LinearProgressIndicator(
              value: _progress,
              backgroundColor: Colors.transparent,
              valueColor: const AlwaysStoppedAnimation<Color>(Colors.blue),
              minHeight: 2,
            ),

          // WebView Content
          Expanded(
            child: ClipRRect(
              child: WebViewWidget(controller: _controller),
            ),
          ),
        ],
      ),
    ),
    ),
    );
  }
}
