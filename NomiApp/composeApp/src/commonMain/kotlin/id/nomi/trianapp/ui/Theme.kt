package id.nomi.trianapp.ui

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

// --- 🎨 Nomi COLOR PALETTE (SvelteKit Web Mirror) ---
val Slate950 = Color(0xFF020617) // Deepest background canvas
val Slate900 = Color(0xFF0F172A) // Main surface card background (#0f172a)
val Slate800 = Color(0xFF1E293B) // Border / stroke colors
val Slate400 = Color(0xFF94A3B8) // Muted text secondary color
val Slate100 = Color(0xFFF1F5F9) // Primary high-contrast text color

val Emerald500 = Color(0xFF10B981) // Success accents / positive flows
val AccentBlue = Color(0xFF38BDF8) // High-performance interactive elements

private val NomiColorScheme = darkColorScheme(
    primary = AccentBlue,
    secondary = Emerald500,
    background = Slate950,
    surface = Slate900,
    onBackground = Slate100,
    onSurface = Slate100,
    outline = Slate800
)

// --- ✍️ TYPOGRAPHY CONFIGURATION ---
val NomiTypography = Typography(
    headlineLarge = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontWeight = FontWeight.Bold,
        fontSize = 32.sp,
        lineHeight = 40.sp,
        color = Slate100
    ),
    titleLarge = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontWeight = FontWeight.SemiBold,
        fontSize = 20.sp,
        lineHeight = 28.sp,
        color = Slate100
    ),
    bodyLarge = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontWeight = FontWeight.Normal,
        fontSize = 16.sp,
        lineHeight = 24.sp,
        color = Slate100
    ),
    bodyMedium = TextStyle(
        fontFamily = FontFamily.SansSerif,
        fontWeight = FontWeight.Normal,
        fontSize = 14.sp,
        lineHeight = 20.sp,
        color = Slate400
    ),
    labelSmall = TextStyle(
        fontFamily = FontFamily.Monospace, // Monospace accents for tracking codes/logs
        fontWeight = FontWeight.Medium,
        fontSize = 12.sp,
        lineHeight = 16.sp,
        color = AccentBlue
    )
)

// --- 🚀 THEME WRAPPER COMPOSABLE ---
@Composable
fun NomiTheme(
    content: @Composable () -> Unit
) {
    MaterialTheme(
        colorScheme = NomiColorScheme,
        typography = NomiTypography,
        content = content
    )
}