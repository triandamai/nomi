package id.nomi.trianapp.ui.screen.admin

import androidx.compose.animation.*
import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.composables.icons.lucide.*
import id.nomi.trianapp.data.model.AdminUserDto
import id.nomi.trianapp.ui.*
import org.koin.compose.viewmodel.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun UserManagementPage(
    viewModel: UserManagementViewModel = koinViewModel()
) {
    val uiState by viewModel.uiState.collectAsState()
    val sheetState = rememberModalBottomSheetState()
    var showBottomSheet by remember { mutableStateOf(false) }

    LaunchedEffect(uiState.selectedUserDetail) {
        if (uiState.selectedUserDetail != null) {
            showBottomSheet = true
        }
    }

    Box(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(24.dp)
        ) {
            Text(
                "User Management",
                color = Color.White,
                fontSize = 24.sp,
                fontWeight = FontWeight.Bold
            )
            Spacer(modifier = Modifier.height(20.dp))

            if (uiState.isLoading && uiState.users.isEmpty()) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator(color = Indigo500)
                }
            } else if (uiState.error != null && uiState.users.isEmpty()) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text(uiState.error!!, color = Color.Red)
                }
            } else {
                LazyColumn(
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                    modifier = Modifier.fillMaxSize()
                ) {
                    items(uiState.users) { user ->
                        UserItem(
                            user = user,
                            isSelected = uiState.selectedUserDetail?.user?.id == user.id,
                            onClick = { viewModel.fetchUserDetail(user.id) }
                        )
                    }
                }
            }
        }

        // Bottom Sheet for Detail
        if (showBottomSheet && (uiState.selectedUserDetail != null || uiState.isDetailLoading)) {
            ModalBottomSheet(
                onDismissRequest = {
                    showBottomSheet = false
                    viewModel.clearSelectedUser()
                },
                sheetState = sheetState,
                containerColor = Slate950,
            ) {
                UserDetailContent(uiState = uiState)
            }
        }
    }
}

@Composable
fun UserItem(
    user: AdminUserDto,
    isSelected: Boolean,
    onClick: () -> Unit
) {
    Surface(
        color = if (isSelected) Slate800 else Slate900,
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
            .border(
                width = 1.dp,
                color = if (isSelected) Indigo500 else Color.Transparent,
                shape = RoundedCornerShape(12.dp)
            )
    ) {
        Row(
            modifier = Modifier.padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(CircleShape)
                    .background(Slate800),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = (user.displayName.firstOrNull() ?: "?").toString().uppercase(),
                    color = Color.White,
                    fontWeight = FontWeight.Bold
                )
            }
            Spacer(modifier = Modifier.width(16.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    user.displayName,
                    color = Color.White,
                    fontSize = 16.sp,
                    fontWeight = FontWeight.SemiBold
                )
                Text(
                    user.role,
                    color = Slate400,
                    fontSize = 13.sp
                )
            }
            if (user.isVerified) {
                Icon(
                    Lucide.Check,
                    contentDescription = "Verified",
                    tint = Indigo500,
                    modifier = Modifier.size(16.dp)
                )
            }
            Spacer(modifier = Modifier.width(8.dp))
            Icon(
                Lucide.ChevronRight,
                contentDescription = null,
                tint = Slate400,
                modifier = Modifier.size(16.dp)
            )
        }
    }
}

@Composable
fun UserDetailContent(
    uiState: UserManagementState
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 24.dp)
            .padding(bottom = 32.dp)
    ) {
        Text(
            "User Detail",
            color = Color.White,
            fontSize = 20.sp,
            fontWeight = FontWeight.Bold
        )
        Spacer(modifier = Modifier.height(16.dp))
        HorizontalDivider(color = Slate800)
        Spacer(modifier = Modifier.height(24.dp))

        if (uiState.isDetailLoading) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(200.dp),
                contentAlignment = Alignment.Center
            ) {
                CircularProgressIndicator(color = Indigo500)
            }
        } else if (uiState.selectedUserDetail != null) {
            val detail = uiState.selectedUserDetail
            LazyColumn(
                verticalArrangement = Arrangement.spacedBy(24.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                // Profile Info
                item {
                    DetailSection("Profile Information") {
                        DetailItem("UUID", detail.user.id)
                        DetailItem("Display Name", detail.user.displayName)
                        DetailItem("Email", detail.user.email ?: "-")
                        DetailItem("Role", detail.user.role)
                        DetailItem("Verified", if (detail.user.isVerified) "Yes" else "No")
                        DetailItem("Created At", detail.user.createdAt)
                    }
                }

                // Channels
                item {
                    DetailSection("Linked Channels") {
                        if (detail.channels.isEmpty()) {
                            Text("No channels linked", color = Slate400, fontSize = 14.sp)
                        } else {
                            detail.channels.forEach { channel ->
                                ChannelItem(channel)
                                Spacer(modifier = Modifier.height(8.dp))
                            }
                        }
                    }
                }

                // Conversations
                item {
                    DetailSection("Conversations") {
                        if (detail.conversations.isEmpty()) {
                            Text("No conversations found", color = Slate400, fontSize = 14.sp)
                        } else {
                            detail.conversations.forEach { conversation ->
                                ConversationDetailItem(conversation)
                                Spacer(modifier = Modifier.height(8.dp))
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun DetailSection(title: String, content: @Composable () -> Unit) {
    Column {
        Text(
            title,
            color = Indigo500,
            fontSize = 14.sp,
            fontWeight = FontWeight.Bold,
            modifier = Modifier.padding(bottom = 12.dp)
        )
        content()
    }
}

@Composable
fun DetailItem(label: String, value: String) {
    Column(modifier = Modifier.padding(bottom = 12.dp)) {
        Text(label, color = Slate400, fontSize = 12.sp)
        Text(value, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.Medium)
    }
}

@Composable
fun ChannelItem(channel: id.nomi.trianapp.data.model.AdminChannelDto) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(Slate800, RoundedCornerShape(8.dp))
            .padding(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            when (channel.channelType.lowercase()) {
                "whatsapp" -> Lucide.MessageCircle
                "telegram" -> Lucide.Send
                else -> Lucide.Hash
            },
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier.size(20.dp)
        )
        Spacer(modifier = Modifier.width(12.dp))
        Column {
            Text(channel.channelType.uppercase(), color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.Bold)
            Text(channel.externalId, color = Slate400, fontSize = 12.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

@Composable
fun ConversationDetailItem(conversation: id.nomi.trianapp.data.model.AdminConversationDto) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(Slate800, RoundedCornerShape(8.dp))
            .padding(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Icon(
            Lucide.MessageSquare,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier.size(20.dp)
        )
        Spacer(modifier = Modifier.width(12.dp))
        Column {
            Text(conversation.title, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.Bold)
            Text("Joined: ${conversation.joinedAt.split("T").firstOrNull() ?: ""}", color = Slate400, fontSize = 12.sp)
        }
    }
}
