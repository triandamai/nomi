package id.nomi.trianapp.ui.screen.workspace

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import id.nomi.trianapp.MainViewModel
import id.nomi.trianapp.data.local.ConversationEntity
import id.nomi.trianapp.ui.Slate800
import id.nomi.trianapp.ui.Slate900
import id.nomi.trianapp.ui.Slate950
import org.koin.compose.viewmodel.koinViewModel
import com.composables.icons.lucide.*
import id.nomi.trianapp.MainAppState
import id.nomi.trianapp.ui.Indigo500
import id.nomi.trianapp.ui.Slate400
import id.nomi.trianapp.ui.screen.Route

enum class WorkspaceMenu {
    Conversations, Reminders, MoneyTracking, Channel, ConversationMonitoring, UserDirectory, Storage, PubSubTest, Health, SoulTimeline
}

@Composable
fun WorkspacePage(
    viewModel: WorkspaceViewModel = koinViewModel(),
    onConversationSelected: (String) -> Unit = {}
) {
    val state by viewModel.state.collectAsState()
    val conversations by viewModel.conversations.collectAsState()
    val profile by viewModel.profile.collectAsState()
    var selectedMenu by remember { mutableStateOf(WorkspaceMenu.Conversations) }

    LaunchedEffect(Unit) {
        viewModel.fetchConversations()
    }
    Row(
        modifier = Modifier.fillMaxSize()
    ) {
        Sidebar(
            currentRoute = Route.Workspace,
            onNavigate = { route ->

            }
        )
        // Crisp vertical divider mimicking web aesthetics
        VerticalDivider(
            color = MaterialTheme.colorScheme.outline.copy(alpha = 0.1f),
            thickness = 0.1.dp
        )

        Column(modifier = Modifier.weight(1f).fillMaxHeight()) {
            // Main Content
            Box(modifier = Modifier.fillMaxHeight().background(Slate950)) {
                when (selectedMenu) {
                    WorkspaceMenu.Conversations -> {
                        ConversationList(
                            conversations = conversations,
                            isLoading = state is WorkspaceState.Loading,
                            error = (state as? WorkspaceState.Error)?.message,
                            onConversationClick = { id ->
                                viewModel.selectConversation(id)
                                onConversationSelected(id)
                            }
                        )
                    }

                    else -> {
                        Box(
                            modifier = Modifier.fillMaxSize(),
                            contentAlignment = Alignment.Center
                        ) {
                            Text(selectedMenu.name, color = Color.White, fontSize = 24.sp)
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun ConversationList(
    conversations: List<ConversationEntity>,
    isLoading: Boolean,
    error: String?,
    onConversationClick: (String) -> Unit = {}
) {
    Column(modifier = Modifier.fillMaxSize().padding(24.dp)) {
        Text(
            "Conversations",
            color = Color.White,
            fontSize = 24.sp,
            fontWeight = FontWeight.Bold
        )
        Spacer(modifier = Modifier.height(20.dp))

        if (isLoading && conversations.isEmpty()) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(color = Color.White)
            }
        } else if (error != null && conversations.isEmpty()) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text(error, color = Color.Red)
            }
        } else {
            LazyColumn(
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                items(conversations) { conversation ->
                    ConversationItem(conversation, navigate = {
                        onConversationClick(conversation.id)
                    })
                }
            }
        }
    }
}

@Composable
fun ConversationItem(
    conversation: ConversationEntity,
    navigate: () -> Unit = {}
) {
    Surface(
        color = Slate900,
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier.fillMaxWidth().clickable { navigate() }
    ) {
        Row(
            modifier = Modifier.padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(CircleShape)
                    .background(Slate800),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    Lucide.MessageSquare,
                    contentDescription = null,
                    tint = Color.White,
                    modifier = Modifier.size(24.dp)
                )
            }
            Spacer(modifier = Modifier.width(16.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    conversation.name,
                    color = Color.White,
                    fontSize = 16.sp,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    "${conversation.cumulativeTokens} tokens",
                    color = Color.Gray,
                    fontSize = 13.sp
                )
            }
            Column(horizontalAlignment = Alignment.End) {
                Text(
                    conversation.updatedAt.split("T").firstOrNull() ?: "",
                    color = Color.Gray,
                    fontSize = 12.sp
                )
                Icon(
                    Lucide.ChevronRight,
                    contentDescription = null,
                    tint = Color.DarkGray,
                    modifier = Modifier.size(16.dp)
                )
            }
        }
    }
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

        SidebarNavItem(Lucide.MessageSquare, true) {

        }
        Spacer(modifier = Modifier.height(8.dp))
        SidebarNavItem(Lucide.Bell, false) {

        }
        Spacer(modifier = Modifier.height(8.dp))
        SidebarNavItem(

            Lucide.Wallet,
            false
        ) {

        }
        Spacer(modifier = Modifier.height(8.dp))
        SidebarNavItem(Lucide.Layers, false) {

        }
        Spacer(modifier = Modifier.height(8.dp))
        SidebarNavItem(
            Lucide.Activity,
            false
        ) {

        }
        Spacer(modifier = Modifier.height(8.dp))
        SidebarNavItem(
            Lucide.Users,
            false
        ) {

        }
        Spacer(modifier = Modifier.height(8.dp))
        SidebarNavItem(Lucide.Database, false) {

        }
        Spacer(modifier = Modifier.height(8.dp))
        SidebarNavItem(Lucide.Rss, false) {

        }
        Spacer(modifier = Modifier.height(8.dp))
        SidebarNavItem(Lucide.HeartPulse, false) {

        }
        Spacer(modifier = Modifier.height(8.dp))
        SidebarNavItem(
            Lucide.History,
            false
        ) {

        }

        Divider(
            modifier = Modifier.width(32.dp).padding(vertical = 4.dp),
            color = Slate800,
            thickness = 1.dp
        )
        Spacer(modifier = Modifier.height(8.dp))

        SidebarNavItem(
            icon = Lucide.Settings,
            isActive = currentRoute == Route.Workspace,
            onClick = { onNavigate(Route.Workspace) }
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
