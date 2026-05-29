import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:nomi_mobile/core/theme/nomi_theme.dart';

class ThemeNotifier extends Notifier<NomiTheme> {
  static const String _themeKey = 'nomi_selected_theme';

  @override
  NomiTheme build() {
    _loadTheme();
    return NomiTheme.slateDark; // Default theme
  }

  Future<void> _loadTheme() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final savedThemeId = prefs.getString(_themeKey);
      if (savedThemeId != null) {
        final matchedTheme = NomiTheme.values.firstWhere(
          (t) => t.id == savedThemeId,
          orElse: () => NomiTheme.slateDark,
        );
        state = matchedTheme;
      }
    } catch (e) {
      print('Failed to load NomiTheme from storage: $e');
    }
  }

  Future<void> setTheme(NomiTheme theme) async {
    state = theme;
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(_themeKey, theme.id);
    } catch (e) {
      print('Failed to save NomiTheme to storage: $e');
    }
  }
}

final themeProvider = NotifierProvider<ThemeNotifier, NomiTheme>(() {
  return ThemeNotifier();
});
