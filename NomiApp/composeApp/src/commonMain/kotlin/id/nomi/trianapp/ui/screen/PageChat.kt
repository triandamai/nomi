package id.nomi.trianapp.ui.screen


import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import id.nomi.trianapp.ui.NomiTheme
import id.nomi.trianapp.ui.component.ChatBubble

data class Message(val content: String, val isFromUser: Boolean)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PageChat() {
    val messages = remember {
        mutableStateListOf(
            Message("Hello! How can I help you today?", false),
            Message("I'm looking for some information about the app.", true),
            Message("Sure! What would you like to know?", false)
        )
    }
    var inputText by remember { mutableStateOf("") }

    val scaffoldState = rememberBottomSheetScaffoldState(
        bottomSheetState = rememberStandardBottomSheetState(
            initialValue = SheetValue.PartiallyExpanded
        )
    )

    BottomSheetScaffold(
        scaffoldState = scaffoldState,
        sheetContent = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp)
                    .navigationBarsPadding()
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    TextField(
                        value = inputText,
                        onValueChange = { inputText = it },
                        modifier = Modifier.weight(1f),
                        placeholder = { Text("Type a message...") },
                        colors = TextFieldDefaults.colors(
                            unfocusedContainerColor = Color.Transparent,
                            focusedContainerColor = Color.Transparent
                        )
                    )
                    IconButton(onClick = {
                        if (inputText.isNotBlank()) {
                            messages.add(Message(inputText, true))
                            inputText = ""
                        }
                    }) {
                        // Icon(Icons.Default.Send, contentDescription = "Send")
                    }
                }
                Spacer(modifier = Modifier.height(16.dp))
            }
        },
        sheetPeekHeight = 120.dp,
        sheetDragHandle = { BottomSheetDefaults.DragHandle() },
        sheetContainerColor = MaterialTheme.colorScheme.surfaceVariant,
        containerColor = Color(0xFF020617) // Slate 950 from SvelteKit style
    ) { paddingValues ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
            contentPadding = PaddingValues(top = 16.dp, bottom = 16.dp)
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
fun PreviewChat(){
    NomiTheme {
        PageChat()
    }
}
