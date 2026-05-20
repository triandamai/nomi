package id.nomi.trianapp.ui.screen.chat


import androidx.compose.animation.*
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.material3.HorizontalDivider
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
import id.nomi.trianapp.ui.component.ShimmerChatLoading
import id.nomi.trianapp.util.formatTokenCount
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import org.koin.compose.viewmodel.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PageChat(
    viewModel: ChatViewModel = koinViewModel(),
    onNavigationClick: () -> Unit,
    onShowRAG: () -> Unit
) {
    val activeConversation by viewModel.activeConversation.collectAsState()
    val thought by viewModel.thought.collectAsState()
    val activeTool by viewModel.activeTool.collectAsState()
    val isTyping by viewModel.isTyping.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()

    var inputText by remember { mutableStateOf("") }
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()

    val isAtBottom by remember {
        derivedStateOf {
            val layoutInfo = listState.layoutInfo
            val visibleItemsInfo = layoutInfo.visibleItemsInfo
            if (visibleItemsInfo.isEmpty()) {
                true
            } else {
                // Since we are reversing layout, index 0 is now the bottom of the screen.
                val firstVisibleItem = visibleItemsInfo.first()
                firstVisibleItem.index <= 1 
            }
        }
    }

    val sheetState = rememberModalBottomSheetState(
        skipPartiallyExpanded = false,
        confirmValueChange = {
            if (it == SheetValue.Hidden) false
            else true
        }
    )

    val scaffoldState = rememberBottomSheetScaffoldState(
        bottomSheetState = sheetState
    )

    val handleShowThought: () -> Unit = remember {
        {
            // Do nothing, thought expansion handles its own internal layout
        }
    }

    // Initial composition / Navigation load
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
                                activeConversation?.name ?: "Nomi",
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
                                    activeConversation?.let { "${formatTokenCount(it.cumulativeTokens)} tokens used" }
                                        ?: "Select a conversation",
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
                HorizontalDivider(Modifier, thickness = 0.5.dp, color = Slate800)
            }
        },
        sheetContainerColor = Slate700,
        sheetShape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
        sheetContentColor = Slate700,
        sheetPeekHeight = 120.dp,
        sheetContent = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp))
                    .background(Slate700)
                    .padding(horizontal = 0.dp, vertical = 2.dp)
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(end = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    TextField(
                        value = inputText,
                        onValueChange = { inputText = it },
                        modifier = Modifier.weight(0.85f),
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
                    Box(
                        modifier = Modifier.weight(0.1f)
                    ) {
                        IconButton(
                            onClick = {
                                if (inputText.isNotBlank()) {
                                    viewModel.sendMessage(inputText)
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
        }
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Slate950)
        ) {
            LazyColumn(
                state = listState,
                modifier = Modifier.fillMaxSize(),
                verticalArrangement = Arrangement.Bottom,
                reverseLayout = true,
                contentPadding = PaddingValues(top = 16.dp, bottom = 0.dp)
            ) {
                // Because reverseLayout is true, the BOTTOM of the screen is rendered first.
                // Therefore, we must render spacer/typing/tools FIRST, then the reversed message list.
                
                item {
                    Spacer(
                        modifier = Modifier.height(126.dp)
                    )
                }

                if (isTyping) {
                    item {
                        TypingIndicatorRow()
                    }
                }

                if (activeTool != null) {
                    item {
                        ToolBadgeComponent(activeTool!!)
                    }
                }

                if (thought != null) {
                    item {
                        ThinkingBubbleComponent(thought!!)
                    }
                }

                // Reverse the messages so newest are at index 0
                val reversedMessages = viewModel.messages.reversed()
                itemsIndexed(
                    reversedMessages,
                    key = { idx, item -> item.id }
                ) { idx, message ->
                    // Since it's reversed, the "previous" message temporally is actually at idx + 1
                    val prevMessage = if (idx >= reversedMessages.size - 1) null else reversedMessages[idx + 1]
                    ChatBubble(
                        displayName = message.displayName,
                        content = message.content,
                        role = message.role,
                        showAvatar = prevMessage?.userId != message.userId,
                        totalTokens = message.totalTokens,
                        thought = message.thought,
                        onShowThought = handleShowThought
                    )
                }

                if (!isLoading && viewModel.messages.isEmpty()) {
                    item {
                        Column(
                            modifier = Modifier.fillParentMaxSize(),
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
                }

                if (isLoading && viewModel.messages.isEmpty()) {
                    item {
                        ShimmerChatLoading()
                    }
                }
            }

            AnimatedVisibility(
                visible = !isAtBottom,
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(bottom = 16.dp),
                enter = fadeIn() + scaleIn(),
                exit = fadeOut() + scaleOut()
            ) {
                FloatingActionButton(
                    onClick = {
                        scope.launch {
                            if (listState.layoutInfo.totalItemsCount > 0) {
                                listState.animateScrollToItem(0)
                            }
                        }
                    },
                    containerColor = Slate800,
                    contentColor = Color.White,
                    shape = CircleShape,
                    modifier = Modifier
                        .size(40.dp)
                        .border(1.dp, Indigo500, CircleShape)
                ) {
                    Icon(
                        imageVector = Lucide.ArrowDown,
                        contentDescription = "Scroll to Bottom",
                        tint = Color.White,
                        modifier = Modifier.size(20.dp)
                    )
                }
            }
        }
    }
}


@Composable
fun ThinkingBubbleComponent(thought: String) {
    Row {
        Box(
            modifier = Modifier
                .size(40.dp)
                .clip(CircleShape),
            contentAlignment = Alignment.Center
        ) {}
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
}

@Composable
fun ToolBadgeComponent(toolName: String) {
    Row {
        Box(
            modifier = Modifier
                .size(40.dp)
                .clip(CircleShape),
            contentAlignment = Alignment.Center
        ) {}
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
}

@Composable
fun TypingIndicatorRow() {
    Row {
        Box(
            modifier = Modifier
                .size(40.dp)
                .clip(CircleShape),
            contentAlignment = Alignment.Center
        ) {}
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
}
