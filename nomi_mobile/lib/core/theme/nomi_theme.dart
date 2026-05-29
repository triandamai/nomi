import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

enum NomiTheme {
  slateDark(
    id: 'slate-dark',
    name: 'Slate Dark',
    isDark: true,
    bgMain: 0xFF0b0f19,
    bgHeader: 0xFF0f172a,
    textMain: 0xFFF8FAFC,
    textMuted: 0xFF64748B,
    borderMain: 0xFF1E293B,
    primaryColor: 0xFF3B82F6,
    accentColor: 0xFF10B981,
    slate950: 0xFF020617,
    slate900: 0xFF0F172A,
    slate800: 0xFF1E293B,
    slate700: 0xFF334155,
  ),
  nordFrost(
    id: 'nord-frost',
    name: 'Nord Frost',
    isDark: true,
    bgMain: 0xFF20242C,
    bgHeader: 0xFF2E3440,
    textMain: 0xFFECEFF4,
    textMuted: 0xFF64748B,
    borderMain: 0xFF2E3440,
    primaryColor: 0xFF88C0D0,
    accentColor: 0xFFA3BE8C,
    slate950: 0xFF1A1C23,
    slate900: 0xFF2E3440,
    slate800: 0xFF3B4252,
    slate700: 0xFF434C5E,
  ),
  glassPurple(
    id: 'glass-purple',
    name: 'Glass Purple',
    isDark: true,
    bgMain: 0xFF090616,
    bgHeader: 0xFF110B29,
    textMain: 0xFFF5F3FF,
    textMuted: 0xFF8B79B8,
    borderMain: 0xFF1C1435,
    primaryColor: 0xFFD8B4FE,
    accentColor: 0xFFF472B6,
    slate950: 0xFF05030D,
    slate900: 0xFF110B29,
    slate800: 0xFF1D1445,
    slate700: 0xFF2E2063,
  ),
  amethystVelvet(
    id: 'amethyst-velvet',
    name: 'Amethyst Velvet',
    isDark: true,
    bgMain: 0xFF0F0B18,
    bgHeader: 0xFF181126,
    textMain: 0xFFF3E8FF,
    textMuted: 0xFF9E77ED,
    borderMain: 0xFF2E1C4B,
    primaryColor: 0xFFC084FC,
    accentColor: 0xFFEC4899,
    slate950: 0xFF0A0710,
    slate900: 0xFF181126,
    slate800: 0xFF2B1D45,
    slate700: 0xFF412B6B,
  ),
  crystalLight(
    id: 'crystal-light',
    name: 'Crystal Light',
    isDark: false,
    bgMain: 0xFFF8FAFC,
    bgHeader: 0xFFFFFFFF,
    textMain: 0xFF0F172A,
    textMuted: 0xFF64748B,
    borderMain: 0xFFE2E8F0,
    primaryColor: 0xFF3B82F6,
    accentColor: 0xFF10B981,
    slate950: 0xFFFFFFFF,
    slate900: 0xFFF1F5F9,
    slate800: 0xFFE2E8F0,
    slate700: 0xFFCBD5E1,
  ),
  sakuraBlossom(
    id: 'sakura-blossom',
    name: 'Sakura Blossom',
    isDark: false,
    bgMain: 0xFFFFF5F7,
    bgHeader: 0xFFFFFFFF,
    textMain: 0xFF4C0519,
    textMuted: 0xFF9F1239,
    borderMain: 0xFFFECDD3,
    primaryColor: 0xFFDB2777,
    accentColor: 0xFFFB7185,
    slate950: 0xFFFFFFFF,
    slate900: 0xFFFFF5F7,
    slate800: 0xFFFFE4E6,
    slate700: 0xFFFECDD3,
  ),
  mintMatcha(
    id: 'mint-matcha',
    name: 'Mint Matcha',
    isDark: false,
    bgMain: 0xFFF0FDF4,
    bgHeader: 0xFFFFFFFF,
    textMain: 0xFF042F2E,
    textMuted: 0xFF0F766E,
    borderMain: 0xFFCCFBF1,
    primaryColor: 0xFF0F766E,
    accentColor: 0xFFD97706,
    slate950: 0xFFFFFFFF,
    slate900: 0xFFF0FDF4,
    slate800: 0xFFE6FDF0,
    slate700: 0xFFCCFBF1,
  );

  final String id;
  final String name;
  final bool isDark;
  final int bgMain;
  final int bgHeader;
  final int textMain;
  final int textMuted;
  final int borderMain;
  final int primaryColor;
  final int accentColor;
  final int slate950;
  final int slate900;
  final int slate800;
  final int slate700;

  const NomiTheme({
    required this.id,
    required this.name,
    required this.isDark,
    required this.bgMain,
    required this.bgHeader,
    required this.textMain,
    required this.textMuted,
    required this.borderMain,
    required this.primaryColor,
    required this.accentColor,
    required this.slate950,
    required this.slate900,
    required this.slate800,
    required this.slate700,
  });

  ThemeData toThemeData() {
    final baseBrightness = isDark ? Brightness.dark : Brightness.light;
    final colorScheme = ColorScheme.fromSeed(
      seedColor: Color(primaryColor),
      brightness: baseBrightness,
      primary: Color(primaryColor),
      secondary: Color(accentColor),
      surface: Color(bgHeader),
    );

    final baseTextTheme = baseBrightness == Brightness.dark
        ? ThemeData.dark().textTheme
        : ThemeData.light().textTheme;

    // Apply high-fidelity Typography hierarchy (Poppins, Montserrat, Bebas Neue)
    final textTheme = baseTextTheme.copyWith(
      // Poppins for Body Text
      bodyLarge: GoogleFonts.poppins(textStyle: baseTextTheme.bodyLarge, color: Color(textMain)),
      bodyMedium: GoogleFonts.poppins(textStyle: baseTextTheme.bodyMedium, color: Color(textMain)),
      bodySmall: GoogleFonts.poppins(textStyle: baseTextTheme.bodySmall, color: Color(textMuted)),
      
      // Montserrat for Titles, Action Headers, Buttons, Labels
      titleLarge: GoogleFonts.montserrat(textStyle: baseTextTheme.titleLarge, fontWeight: FontWeight.bold, color: Color(textMain)),
      titleMedium: GoogleFonts.montserrat(textStyle: baseTextTheme.titleMedium, fontWeight: FontWeight.bold, color: Color(textMain)),
      titleSmall: GoogleFonts.montserrat(textStyle: baseTextTheme.titleSmall, fontWeight: FontWeight.w600, color: Color(textMain)),
      labelLarge: GoogleFonts.montserrat(textStyle: baseTextTheme.labelLarge, fontWeight: FontWeight.bold, color: Color(textMain)),
      labelMedium: GoogleFonts.montserrat(textStyle: baseTextTheme.labelMedium, fontWeight: FontWeight.w600, color: Color(textMain)),
      labelSmall: GoogleFonts.montserrat(textStyle: baseTextTheme.labelSmall, color: Color(textMuted)),
      
      // Bebas Neue for display/headlines/logo
      displayLarge: GoogleFonts.bebasNeue(textStyle: baseTextTheme.displayLarge, color: Color(textMain)),
      displayMedium: GoogleFonts.bebasNeue(textStyle: baseTextTheme.displayMedium, color: Color(textMain)),
      displaySmall: GoogleFonts.bebasNeue(textStyle: baseTextTheme.displaySmall, color: Color(textMain)),
      headlineLarge: GoogleFonts.bebasNeue(textStyle: baseTextTheme.headlineLarge, color: Color(textMain)),
      headlineMedium: GoogleFonts.bebasNeue(textStyle: baseTextTheme.headlineMedium, color: Color(textMain)),
      headlineSmall: GoogleFonts.bebasNeue(textStyle: baseTextTheme.headlineSmall, color: Color(textMain)),
    );

    return ThemeData(
      brightness: baseBrightness,
      primaryColor: Color(primaryColor),
      scaffoldBackgroundColor: Color(bgMain),
      cardColor: Color(slate950),
      dividerColor: Color(borderMain),
      colorScheme: colorScheme,
      textTheme: textTheme,
      appBarTheme: AppBarTheme(
        backgroundColor: Color(bgHeader),
        foregroundColor: Color(textMain),
        elevation: 0,
      ),
    );
  }
}
