import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nomi_mobile/ui/pages/chat_page.dart';
import 'package:nomi_mobile/ui/widgets/sidebar.dart';

class MainShell extends ConsumerWidget {
  const MainShell({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final size = MediaQuery.of(context).size;
    final bool isLargeScreen = size.width >= 900;

    return Scaffold(
      body: Row(
        children: [
          if (isLargeScreen) const NomiSidebar(isDrawer: false),
          const Expanded(child: ChatPage()),
        ],
      ),
      drawer: isLargeScreen ? null : const NomiSidebar(isDrawer: true),
    );
  }
}
