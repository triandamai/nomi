package id.nomi.trianapp.ui.screen.chat


import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.composables.icons.lucide.*
import id.nomi.trianapp.ui.*
import id.nomi.trianapp.ui.component.ChatBubble
import org.koin.compose.viewmodel.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PageChat(
    viewModel: ChatViewModel = koinViewModel(),
    onNavigationClick: () -> Unit,
    onShowRAG:()-> Unit
) {
    val messages by viewModel.messages.collectAsState()
    val activeConversationId by viewModel.activeConversationId.collectAsState()
    val thought by viewModel.thought.collectAsState()
    val activeTool by viewModel.activeTool.collectAsState()
    val isTyping by viewModel.isTyping.collectAsState()

    var inputText by remember { mutableStateOf("") }
    val listState = rememberLazyListState()


    val sheetState = rememberModalBottomSheetState(
        skipPartiallyExpanded = false
    )

    val scaffoldState = rememberBottomSheetScaffoldState(
        bottomSheetState = sheetState
    )

    LaunchedEffect(messages.size) {
        if (messages.isNotEmpty()) {
            listState.animateScrollToItem(messages.size - 1)
        }
    }
    LaunchedEffect(viewModel) {
        sheetState.show()
    }

    BottomSheetScaffold(
        scaffoldState = scaffoldState,
        sheetDragHandle = {
            Spacer(modifier = Modifier.height(12.dp))
        },
        topBar = {
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
                                        .background(if (activeConversationId != null) Emerald500 else Slate400)
                                )
                                Spacer(modifier = Modifier.width(6.dp))
                                Text(
                                    if (activeConversationId != null) "Active now" else "Select a conversation",
                                    style = MaterialTheme.typography.labelSmall.copy(
                                        color = Slate400,
                                        fontSize = 11.sp,
                                        fontWeight = FontWeight.Normal
                                    )
                                )
                            }
                        }
                    },
                    colors = TopAppBarDefaults.topAppBarColors(
                        containerColor = Slate950,
                        titleContentColor = Color.White,
                    ),
                    navigationIcon = {
                        IconButton(onClick = onNavigationClick) {
                            Icon(
                                imageVector = Lucide.Settings,
                                contentDescription = null,
                                tint = Color.White
                            )
                        }
                    },
                    actions = {
                        IconButton(onClick = onShowRAG) {
                            Icon(
                                imageVector = Lucide.GitGraph,
                                contentDescription = null,
                                tint = Color.White
                            )
                        }
                    }
                )
                Divider(color = Slate800, thickness = 0.5.dp)
            }
        },
        sheetContainerColor = Slate700,
        sheetShape = RoundedCornerShape(24.dp),
        sheetPeekHeight = 120.dp,
        sheetContent = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(24.dp)),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    TextField(
                        value = inputText,
                        onValueChange = { inputText = it },
                        modifier = Modifier.weight(1f),
                        maxLines = 2,
                        minLines = 2,
                        placeholder = {
                            Text(
                                "Type your message...",
                                color = Slate400,
                                fontSize = 15.sp
                            )
                        },
                        colors = TextFieldDefaults.colors(
                            unfocusedContainerColor = Color.Transparent,
                            focusedContainerColor = Color.Transparent,
                            unfocusedIndicatorColor = Color.Transparent,
                            focusedIndicatorColor = Color.Transparent,
                            cursorColor = Indigo500,
                        ),
                        textStyle = MaterialTheme.typography.bodyMedium.copy(
                            color = Color.White,
                            fontSize = 15.sp
                        )
                    )
                    IconButton(
                        onClick = {
                            if (inputText.isNotBlank()) {
                                // Implement send logic if needed, for now just clear
                                inputText = ""
                            }
                        },
                        modifier = Modifier
                            .size(36.dp)
                            .clip(CircleShape)
                            .background(if (inputText.isBlank()) Slate700 else Indigo500),
                        enabled = inputText.isNotBlank()
                    ) {
                        Icon(
                            imageVector = Lucide.Send,
                            contentDescription = "Send",
                            tint = Color.White,
                            modifier = Modifier.size(18.dp)
                        )
                    }
                }
            }
        }
    ) { padding ->
        Box(modifier = Modifier.fillMaxSize().padding(padding).background(Slate950)) {
            if (activeConversationId == null || messages.isEmpty()) {
                EmptyStateComponent()
            } else {
                LazyColumn(
                    state = listState,
                    modifier = Modifier.fillMaxSize(),
                    verticalArrangement = Arrangement.Top,
                    contentPadding = PaddingValues(top = 16.dp, bottom = 24.dp)
                ) {
                    itemsIndexed(messages) { idx,message ->
                        val prevMessage = if(idx <= 0) null else messages[idx - 1]
                        ChatBubble(
                            content = message.content,
                            isFromUser = message.role == "user",
                            showAvatar = prevMessage?.userId !== message.userId
                        )
                    }

                    if (thought != null) {
                        item {
                            ThinkingBubbleComponent(thought!!)
                        }
                    }

                    if (activeTool != null) {
                        item {
                            ToolBadgeComponent(activeTool!!)
                        }
                    }

                    if (isTyping) {
                        item {
                            TypingIndicatorRow()
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun EmptyStateComponent() {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(
            imageVector = Lucide.MessageSquare,
            contentDescription = null,
            tint = Slate400,
            modifier = Modifier.size(64.dp)
        )
        Spacer(modifier = Modifier.height(16.dp))
        Text(
            "No messages yet. Start your conversation with Nomi!",
            color = Slate400,
            textAlign = TextAlign.Center,
            fontSize = 16.sp,
            modifier = Modifier.padding(horizontal = 32.dp)
        )
    }
}

@Composable
fun ThinkingBubbleComponent(thought: String) {
    Surface(
        color = Slate900.copy(alpha = 0.5f),
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp).fillMaxWidth()
    ) {
        Row(modifier = Modifier.padding(12.dp), verticalAlignment = Alignment.Top) {
            Icon(
                Lucide.Brain,
                contentDescription = null,
                tint = Indigo500,
                modifier = Modifier.size(16.dp)
            )
            Spacer(modifier = Modifier.width(8.dp))
            Text(thought, color = Slate400, fontSize = 13.sp, fontWeight = FontWeight.Light)
        }
    }
}

@Composable
fun ToolBadgeComponent(toolName: String) {
    Surface(
        color = Indigo500.copy(alpha = 0.1f),
        border = androidx.compose.foundation.BorderStroke(1.dp, Indigo500.copy(alpha = 0.3f)),
        shape = RoundedCornerShape(8.dp),
        modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(modifier = Modifier.size(6.dp).clip(CircleShape).background(Indigo500))
            Spacer(modifier = Modifier.width(6.dp))
            Text(
                "Running $toolName...",
                color = Indigo500,
                fontSize = 11.sp,
                fontWeight = FontWeight.Medium
            )
        }
    }
}

@Composable
fun TypingIndicatorRow() {
    Row(
        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        repeat(3) { index ->
            Box(
                modifier = Modifier
                    .padding(horizontal = 2.dp)
                    .size(6.dp)
                    .clip(CircleShape)
                    .background(Slate400.copy(alpha = 0.6f))
            )
        }
    }
}
