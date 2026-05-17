package id.nomi.trianapp.ui.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.runtime.rememberNavBackStack
import androidx.navigation3.ui.NavDisplay
import androidx.savedstate.serialization.SavedStateConfiguration
import com.composables.icons.lucide.*
import id.nomi.trianapp.MainAppState
import id.nomi.trianapp.MainViewModel
import id.nomi.trianapp.ui.*
import id.nomi.trianapp.ui.screen.auth.LoginPage
import id.nomi.trianapp.ui.screen.chat.PageChat
import id.nomi.trianapp.ui.screen.profile.ProfilePage
import id.nomi.trianapp.ui.screen.rag.RagPage
import id.nomi.trianapp.ui.screen.setting.SettingPage
import kotlinx.serialization.Serializable
import kotlinx.serialization.modules.SerializersModule
import kotlinx.serialization.modules.polymorphic
import kotlinx.serialization.modules.subclass
import org.koin.compose.viewmodel.koinViewModel

@Serializable
sealed interface Route : NavKey {
    @Serializable
    data object Login : Route

    @Serializable
    data object Chat : Route

    @Serializable
    data object Profile : Route

    @Serializable
    data object Settings : Route

    @Serializable
    data object Rag : Route
}

private val config = SavedStateConfiguration {
    serializersModule = SerializersModule {
        polymorphic(NavKey::class) {
            subclass(Route.Login::class)
            subclass(Route.Chat::class)
            subclass(Route.Profile::class)
            subclass(Route.Settings::class)
            subclass(Route.Rag::class)
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainApp() {
    val mainVm = koinViewModel<MainViewModel>()
    val appState by mainVm.appState.collectAsState()

    val backStack = rememberNavBackStack(config, Route.Login)

    LaunchedEffect(appState) {
        when(appState) {
            MainAppState.Authenticated -> {
                if (backStack.last() == Route.Login) {
                    backStack.add(Route.Chat)
                }
            }
            MainAppState.Unauthenticated -> {
                if (backStack.last() != Route.Login) {
                    backStack.add(Route.Login)
                }
            }
            else -> {}
        }
    }

    NavDisplay(
        backStack = backStack,
        onBack = { if (backStack.size > 1) backStack.removeLast() },
        entryProvider = entryProvider {
            entry<Route.Login> {
                LoginPage()
            }
            entry<Route.Chat> {
                PageChat(onNavigationClick = {
                    if (backStack.last() != Route.Settings) {
                        backStack.add(Route.Settings)
                    }
                })
            }
            entry<Route.Profile> {
                ProfilePage()
            }
            entry<Route.Settings> {
                Row(
                    modifier = Modifier.fillMaxSize()
                ) {
                    if (appState is MainAppState.Authenticated) {
                        Sidebar(
                            currentRoute = Route.Chat,
                            onNavigate = { route ->
                                if (backStack.last() != route) {
                                    backStack.add(route)
                                }
                            }
                        )
                        // Crisp vertical divider mimicking web aesthetics
                        VerticalDivider(
                            color = MaterialTheme.colorScheme.outline.copy(alpha = 0.1f),
                            thickness = 0.5.dp
                        )
                    }
                    Column(modifier = Modifier.weight(1f).fillMaxHeight()) {
                        Column(modifier = Modifier.background(Slate950)) {
                            CenterAlignedTopAppBar(
                                title = {
                                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                        Text(
                                            "Nomi",
                                            style = MaterialTheme.typography.titleLarge.copy(
                                                fontWeight = FontWeight.SemiBold,
                                                fontSize = 17.sp,
                                                letterSpacing = 0.sp
                                            )
                                        )
                                        Row(
                                            verticalAlignment = Alignment.CenterVertically,
                                            horizontalArrangement = Arrangement.Center
                                        ) {
                                            Box(
                                                modifier = Modifier
                                                    .size(6.dp)
                                                    .clip(CircleShape)
                                                    .background(Emerald500)
                                            )
                                            Spacer(modifier = Modifier.width(6.dp))
                                            Text(
                                                "Active now",
                                                style = MaterialTheme.typography.labelSmall.copy(
                                                    color = Slate400,
                                                    fontSize = 11.sp,
                                                    fontWeight = FontWeight.Normal
                                                )
                                            )
                                        }
                                    }
                                },
                                colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
                                    containerColor = Slate950,
                                    titleContentColor = Color.White
                                )
                            )
                            Divider(color = Slate800, thickness = 0.5.dp)
                        }
                        SettingPage()
                    }
                }
            }
            entry<Route.Rag> {
                RagPage()
            }
        }
    )
}

@Composable
fun Sidebar(currentRoute: Route, onNavigate: (Route) -> Unit) {
    Column(
        modifier = Modifier
            .width(72.dp)
            .fillMaxHeight()
            .background(Slate950)
            .padding(vertical = 12.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Box(
            modifier = Modifier.size(48.dp)
        ) {

        }
        // App/Home Icon
        SidebarNavItem(
            icon = Lucide.House,
            isActive = currentRoute == Route.Chat,
            onClick = { onNavigate(Route.Chat) }
        )

        Spacer(modifier = Modifier.height(8.dp))
        Divider(
            modifier = Modifier.width(32.dp).padding(vertical = 4.dp),
            color = Slate800,
            thickness = 1.dp
        )
        Spacer(modifier = Modifier.height(8.dp))

        // Navigation Items
        SidebarNavItem(
            icon = Lucide.BookOpen,
            isActive = currentRoute == Route.Rag,
            onClick = { onNavigate(Route.Rag) }
        )
        SidebarNavItem(
            icon = Lucide.User,
            isActive = currentRoute == Route.Profile,
            onClick = { onNavigate(Route.Profile) }
        )

        Spacer(modifier = Modifier.weight(1f))

        SidebarNavItem(
            icon = Lucide.Settings,
            isActive = currentRoute == Route.Settings,
            onClick = { onNavigate(Route.Settings) }
        )
    }
}

@Composable
fun SidebarNavItem(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    isActive: Boolean,
    onClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .padding(vertical = 4.dp)
            .size(48.dp)
            .clip(if (isActive) RoundedCornerShape(16.dp) else CircleShape)
            .background(if (isActive) Indigo500 else Color.Transparent)
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = if (isActive) Color.White else Slate400,
            modifier = Modifier.size(24.dp)
        )
    }
}

@Preview
@Composable
fun MainAppPreview() {
    NomiTheme {
        MainApp()
    }
}
