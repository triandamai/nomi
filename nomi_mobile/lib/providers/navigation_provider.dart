import 'package:flutter_riverpod/flutter_riverpod.dart';

enum MainView {
  chat,
  storage,
  reinforcement,
  guardrails,
  skills,
  monitor,
  pluginEditor,
}

class NavigationState {
  final MainView activeView;
  final Map<String, dynamic>? arguments;

  NavigationState({required this.activeView, this.arguments});

  NavigationState copyWith({MainView? activeView, Map<String, dynamic>? arguments}) {
    return NavigationState(
      activeView: activeView ?? this.activeView,
      arguments: arguments ?? this.arguments,
    );
  }
}

class NavigationNotifier extends Notifier<NavigationState> {
  @override
  NavigationState build() {
    return NavigationState(activeView: MainView.chat);
  }

  void navigateTo(MainView view, {Map<String, dynamic>? args}) {
    state = state.copyWith(activeView: view, arguments: args);
  }
}

final navigationProvider = NotifierProvider<NavigationNotifier, NavigationState>(() {
  return NavigationNotifier();
});
