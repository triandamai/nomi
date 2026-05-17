package id.nomi.trianapp.ui.screen.chat


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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.composables.icons.lucide.*
import id.nomi.trianapp.ui.*
import id.nomi.trianapp.ui.component.ChatBubble

data class Message(val content: String, val isFromUser: Boolean)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PageChat(
    onNavigationClick: () -> Unit
) {
    val messages = remember {
        mutableStateListOf(
            Message("Hello! I'm Nomi. How can I help you today?", false),
            Message("I want to know more about the workspace features.", true),
            Message(
                "The workspace provides a suite of tools for AI-driven development, including real-time streaming, vector search, and integrated artifacts.",
                false
            )
        )
    }
    var inputText by remember { mutableStateOf("") }

    val sheetState = rememberModalBottomSheetState(
        skipPartiallyExpanded = false,
        confirmValueChange = {
            true
        }
    )

    val scaffoldState = rememberBottomSheetScaffoldState(
        bottomSheetState = sheetState
    )
    LaunchedEffect(scaffoldState) {
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
                    colors = TopAppBarDefaults.topAppBarColors(
                        containerColor = Slate950,
                        scrolledContainerColor = Color.Unspecified,
                        navigationIconContentColor = Color.Unspecified,
                        titleContentColor = Color.White,
                        actionIconContentColor = Color.Unspecified
                    ),
                    navigationIcon = {
                        IconButton(
                            onClick = {
                                onNavigationClick()
                            }
                        ) {
                            Icon(
                                imageVector = Lucide.Settings,
                                contentDescription = null
                            )
                        }
                    }
                )
                Divider(color = Slate800, thickness = 0.5.dp)
            }
        },
        sheetContent = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp)
                    .padding(bottom = 32.dp) // Extra padding for flush look
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(24.dp))
                        .padding(horizontal = 12.dp, vertical = 4.dp),
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
                                messages.add(Message(inputText, true))
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
        },
        sheetSwipeEnabled = true,
        sheetPeekHeight = 120.dp
    ) {
        LazyColumn(
            modifier = Modifier
                .fillMaxSize(),
            verticalArrangement = Arrangement.Top,
            contentPadding = PaddingValues(top = 16.dp, bottom = 24.dp)
        ) {
            items(messages) { message ->
                ChatBubble(
                    content = message.content,
                    isFromUser = message.isFromUser
                )
            }
        }
    }

}

@Preview
@Composable
fun PageChatPreview() {
    NomiTheme {
        PageChat(onNavigationClick = {})
    }
}
