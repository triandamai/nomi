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
import id.nomi.trianapp.ui.screen.workspace.WorkspacePage
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
    data object Workspace : Route

    @Serializable
    data object Rag : Route
}

private val config = SavedStateConfiguration {
    serializersModule = SerializersModule {
        polymorphic(NavKey::class) {
            subclass(Route.Login::class)
            subclass(Route.Chat::class)
            subclass(Route.Profile::class)
            subclass(Route.Workspace::class)
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
        when (appState) {
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
                    if (backStack.last() != Route.Workspace) {
                        backStack.add(Route.Workspace)
                    }
                }, onShowRAG = {
                    if (backStack.last() != Route.Rag) {
                        backStack.add(Route.Rag)
                    }
                })
            }
            entry<Route.Profile> {
                ProfilePage()
            }
            entry<Route.Workspace> {
                WorkspacePage(onConversationSelected = {
                    if (backStack.last() != Route.Chat) {
                        backStack.add(Route.Chat)
                    }
                })
            }
            entry<Route.Rag> {
                RagPage(
                    onNavigationClick = {
                        if (backStack.last() == Route.Rag) {
                            backStack.removeLast()
                        }
                    }
                )
            }
        }
    )
}

@Preview
@Composable
fun MainAppPreview() {
    NomiTheme {
        MainApp()
    }
}
