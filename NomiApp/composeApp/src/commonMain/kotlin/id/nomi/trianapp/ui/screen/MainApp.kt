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
import id.nomi.trianapp.ui.screen.SplashPage
import id.nomi.trianapp.ui.screen.chat.ChatViewModel
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
    data object Splash : Route

    @Serializable
    data object Login : Route

    @Serializable
    data class Chat(val conversationId: String? = null) : Route

    @Serializable
    data object Profile : Route

    @Serializable
    data object Workspace : Route

    @Serializable
    data class Rag(val conversationId: String? = null) : Route
}

private val config = SavedStateConfiguration {
    serializersModule = SerializersModule {
        polymorphic(NavKey::class) {
            subclass(Route.Splash::class)
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

    val backStack = rememberNavBackStack(config, Route.Splash)

    NavDisplay(
        backStack = backStack,
        onBack = { if (backStack.size > 1) backStack.removeLast() },
        entryProvider = entryProvider {
            entry<Route.Splash> {
                SplashPage(
                    appState = appState,
                    onNavigateToLogin = {
                        backStack.add(Route.Login)
                    },
                    onNavigateToChat = {
                        backStack.add(Route.Chat())
                    }
                )
            }
            entry<Route.Login> {
                LoginPage(onPairingSuccess = {
                    mainVm.checkAuthentication()
                    backStack.add(Route.Chat())
                })
            }
            entry<Route.Chat> { route ->
                val viewModel = koinViewModel<ChatViewModel>()
                LaunchedEffect(route) {
                    val conversationId = route.conversationId
                    if (conversationId != null) {
                        viewModel.setConversationId(conversationId)
                    }else{
                        viewModel.resetConversation()
                    }
                }

                PageChat(
                    viewModel = viewModel,
                    onNavigationClick = {
                        backStack.add(Route.Workspace)
                    },
                    onShowRAG = {
                        backStack.add(Route.Rag(route.conversationId))
                    }
                )
            }
            entry<Route.Profile> {
                ProfilePage()
            }
            entry<Route.Workspace> {
                WorkspacePage(onConversationSelected = { id ->
                    backStack.add(Route.Chat(id))
                })
            }
            entry<Route.Rag> { route ->
                RagPage(
                    conversationId = route.conversationId,
                    onNavigationClick = {
                        backStack.removeLast()
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
