

### 🤖 Mission: Nomi Mobile Full Lifecycle Implementation


 I need to build the complete mobile application for **Nomi** from scratch using the modern Kotlin Multiplatform (KMM) structure centered around the **`composeApp`** directory. The application must feature an identical dark-themed aesthetic, styling, and seamless UX patterns as our SvelteKit web dashboard.
 **Phase 1: Project Initialization & Directory Layout**
 * Initialize a modern KMM template the project name is `NomiApp`. All shared application code, UI frameworks, and business logic must reside within the `composeApp/src/commonMain/` directory. Platform-specific configurations must reside in `androidMain/` and `iosMain/`.


 **Phase 2: Build Configurations & Dependencies**
 * Configure `composeApp/build.gradle.kts` with the following foundation:
 * **DI:** `io.insert-koin:koin-core` and `io.insert-koin:koin-compose`
 * **Networking:** `io.ktor:ktor-client-core`, `io.ktor:ktor-client-content-negotiation`, and JSON serialization plugins.
 * **Asynchronous Execution:** Kotlin Coroutines (`kotlinx-coroutines-core`).
 * **Android SDK Architecture:** Jetpack WorkManager (`androidx-work:work-runtime-ktx`) configured inside the Android target source sets.




 **Phase 3: UI Architecture (SvelteKit Styling Mirror)**
 * Implement the layout system in `commonMain` using **Compose Multiplatform**.
 * **Design Palette:** Enforce a strict dark-mode theme utilizing a Slate-900 surface canvas (`#0f172a`), sleek thin borders, and soft corner radiuses matching our Tailwind CSS configurations.
 * **Components:**
 * A high-fidelity main dashboard layout with a real-time 'Total Spent' banner.
 * An interactive **Transaction History PopUp overlay** with an inline search bar, quick emoji filtering chips (🍔, ⛽, 🏔️), CRUD update states, and dynamic pagination that activates when rows exceed 20 items.




 **Phase 4: Business & State Logic**
 * Build a unified State Management architecture in `commonMain` utilizing Kotlin `StateFlow` to drive UI rendering. This must mimic our reactive Svelte store patterns, enabling immediate **Optimistic UI updates** during transaction deletion or value modifications before the API network handshake confirms success.
 * Setup Koin dependency injection modules (`di/Koin.kt`) to cleanly instantiate clients and viewmodels.


 **Phase 5: Full-Stack Backend Integration (REST + SSE)**
 * Implement the Ktor network layer client (`network/NomiClient.kt`).
 * Build out complete resource handlers for `GET`, `PATCH`, and `DELETE` actions pointing to our Rust Gateway endpoints.
 * Implement a persistent Server-Sent Events (SSE) background stream for pushing instantaneous transaction updates directly from the server to the mobile UI canvas without resource-heavy polling hooks.


 **Phase 6: Health Connect SDK Integration**
 * In `androidMain`, hook into the **Android Health Connect SDK**.
 * Set up strict data mapping wrappers to extract data streams from a **Galaxy Fit 3** via Samsung Health, specifically capturing: `StepsRecord`, `SleepSessionRecord`, `HeartRateRecord`, and `TotalCaloriesBurnedRecord`.


 **Phase 7: Background Dispatch Workers**
 * Build a robust, power-optimized `PeriodicWorkRequest` using Android's native **WorkManager**.
 * Configure the schedule to run automatically every 4 to 6 hours with system constraints requiring an active network connection. The worker must safely batch-compile the latest aggregated Galaxy Fit 3 biometric data blocks from Health Connect and securely dispatch them to `POST /v1/health/sync` on our Rust Gateway backend."


Theme reference:
```kt
package ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.Typography
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
```