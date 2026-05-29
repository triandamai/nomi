import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nomi_mobile/providers/navigation_provider.dart';
import 'package:nomi_mobile/providers/chat_provider.dart';
import 'package:nomi_mobile/ui/pages/chat_page.dart';
import 'package:nomi_mobile/ui/pages/storage_page.dart';
import 'package:nomi_mobile/ui/pages/reinforcement_page.dart';
import 'package:nomi_mobile/ui/pages/guardrail_page.dart';
import 'package:nomi_mobile/ui/pages/skills_page.dart';
import 'package:nomi_mobile/ui/pages/monitor_page.dart';
import 'package:nomi_mobile/ui/pages/plugin_editor.dart';
import 'package:nomi_mobile/ui/widgets/sidebar.dart';
import 'package:nomi_mobile/data/models/plugin.dart';

class MainShell extends ConsumerWidget {
  const MainShell({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final navState = ref.watch(navigationProvider);
    final chatState = ref.watch(chatProvider);
    final size = MediaQuery.of(context).size;
    final bool isLargeScreen = size.width >= 900;

    Widget body;
    switch (navState.activeView) {
      case MainView.chat:
        body = const ChatPage();
        break;
      case MainView.storage:
        body = const StoragePage();
        break;
      case MainView.reinforcement:
        body = const ReinforcementPage();
        break;
      case MainView.guardrails:
        body = const GuardrailPage();
        break;
      case MainView.skills:
        body = const SkillsPage();
        break;
      case MainView.monitor:
        body = const MonitorPage();
        break;
      case MainView.pluginEditor:
        final plugin = navState.arguments?['plugin'] as Plugin;
        body = PluginEditorPage(plugin: plugin);
        break;
    }

    return Scaffold(
      backgroundColor: Theme.of(context).scaffoldBackgroundColor,
      body: isLargeScreen
          ? Stack(
              children: [
                Positioned.fill(
                  child: Padding(
                    padding: const EdgeInsets.only(left: 72),
                    child: body,
                  ),
                ),
                if (chatState.isSidebarExpanded)
                  Positioned.fill(
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTap: () => ref.read(chatProvider.notifier).toggleSidebar(),
                      child: Container(
                        color: Colors.transparent,
                      ),
                    ),
                  ),
                const Positioned(
                  top: 0,
                  bottom: 0,
                  left: 0,
                  child: NomiSidebar(isDrawer: false),
                ),
              ],
            )
          : body,
      drawer: isLargeScreen ? null : const NomiSidebar(isDrawer: true),
    );
  }
}
