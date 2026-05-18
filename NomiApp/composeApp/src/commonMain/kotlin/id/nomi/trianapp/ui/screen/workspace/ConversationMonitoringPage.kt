package id.nomi.trianapp.ui.screen.workspace

import androidx.compose.animation.*
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.composables.icons.lucide.*
import id.nomi.trianapp.data.model.ConversationDto
import id.nomi.trianapp.ui.*
import id.nomi.trianapp.ui.screen.admin.DetailItem
import id.nomi.trianapp.ui.screen.admin.DetailSection
import id.nomi.trianapp.util.NumberSeparatorTransformation
import id.nomi.trianapp.util.formatTokenCount
import org.koin.compose.viewmodel.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConversationMonitoringPage(
    viewModel: ConversationMonitoringViewModel = koinViewModel()
) {
    val uiState by viewModel.uiState.collectAsState()
    var selectedConversation by remember { mutableStateOf<ConversationDto?>(null) }
    val sheetState = rememberModalBottomSheetState()
    var showBottomSheet by remember { mutableStateOf(false) }

    LaunchedEffect(selectedConversation) {
        if (selectedConversation != null) {
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
                "Conversation Monitoring",
                color = Color.White,
                fontSize = 24.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                "Monitor token usage and manage conversation limits",
                color = Slate400,
                fontSize = 14.sp
            )
            Spacer(modifier = Modifier.height(24.dp))

            if (uiState.isLoading && uiState.conversations.isEmpty()) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator(color = Indigo500)
                }
            } else if (uiState.error != null && uiState.conversations.isEmpty()) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text(uiState.error!!, color = Color.Red)
                }
            } else {
                LazyColumn(
                    verticalArrangement = Arrangement.spacedBy(16.dp),
                    modifier = Modifier.fillMaxSize()
                ) {
                    items(uiState.conversations) { conversation ->
                        ConversationUsageItem(
                            conversation = conversation,
                            onClick = { selectedConversation = conversation }
                        )
                    }
                }
            }
        }

        if (showBottomSheet && selectedConversation != null) {
            ModalBottomSheet(
                onDismissRequest = {
                    showBottomSheet = false
                    selectedConversation = null
                },
                sheetState = sheetState,
                containerColor = Slate950
            ) {
                ConversationDetailBottomSheet(
                    conversation = selectedConversation!!,
                    isUpdating = uiState.isUpdating,
                    onUpdateMaxTokens = { maxTokens ->
                        viewModel.updateMaxTokens(selectedConversation!!.id, maxTokens)
                        showBottomSheet = false
                        selectedConversation = null
                    }
                )
            }
        }
    }
}

@Composable
fun ConversationUsageItem(
    conversation: ConversationDto,
    onClick: () -> Unit
) {
    val percentage = if (conversation.maxTokenUsage > 0) {
        (conversation.cumulativeTokens.toFloat() / conversation.maxTokenUsage.toFloat()).coerceIn(0f, 1f)
    } else {
        0f
    }
    
    val progressColor = when {
        percentage > 0.9f -> Color.Red
        percentage > 0.7f -> Color.Yellow
        else -> Indigo500
    }

    Surface(
        color = Slate900,
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Box(
                        modifier = Modifier
                            .size(32.dp)
                            .clip(CircleShape)
                            .background(Slate800),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(Lucide.MessageSquare, contentDescription = null, tint = Color.White, modifier = Modifier.size(16.dp))
                    }
                    Spacer(modifier = Modifier.width(12.dp))
                    Text(
                        conversation.name,
                        color = Color.White,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
                
                Text(
                    "${(percentage * 100).toInt()}%",
                    color = progressColor,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Bold
                )
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            LinearProgressIndicator(
                progress = { percentage },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(8.dp)
                    .clip(RoundedCornerShape(4.dp)),
                color = progressColor,
                trackColor = Slate800
            )
            
            Spacer(modifier = Modifier.height(12.dp))
            
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Text(
                    "Usage: ${formatTokenCount(conversation.cumulativeTokens)}",
                    color = Slate400,
                    fontSize = 12.sp
                )
                Text(
                    "Limit: ${formatTokenCount(conversation.maxTokenUsage)}",
                    color = Slate400,
                    fontSize = 12.sp
                )
            }
        }
    }
}

@Composable
fun ConversationDetailBottomSheet(
    conversation: ConversationDto,
    isUpdating: Boolean,
    onUpdateMaxTokens: (Long) -> Unit
) {
    var maxTokensText by remember { mutableStateOf(conversation.maxTokenUsage.toString()) }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 24.dp)
            .padding(bottom = 32.dp)
    ) {
        Text(
            "Manage Conversation",
            color = Color.White,
            fontSize = 20.sp,
            fontWeight = FontWeight.Bold
        )
        Text(
            conversation.name,
            color = Slate400,
            fontSize = 14.sp
        )
        
        Spacer(modifier = Modifier.height(24.dp))
        HorizontalDivider(color = Slate800)
        Spacer(modifier = Modifier.height(24.dp))

        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            DetailItem("Current Usage", formatTokenCount(conversation.cumulativeTokens))
            DetailItem("Current Limit", formatTokenCount(conversation.maxTokenUsage))
        }

        Spacer(modifier = Modifier.height(24.dp))
        
        DetailSection("Update Limit") {
            Text(
                "Adjust the maximum token limit for this conversation. Setting it to 0 means no limit.",
                color = Slate400,
                fontSize = 13.sp,
                modifier = Modifier.padding(bottom = 16.dp)
            )
            
            OutlinedTextField(
                value = maxTokensText,
                onValueChange = { if (it.all { char -> char.isDigit() }) maxTokensText = it },
                label = { Text("Max Tokens") },
                modifier = Modifier.fillMaxWidth(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                visualTransformation = NumberSeparatorTransformation(),
                colors = OutlinedTextFieldDefaults.colors(
                    focusedTextColor = Color.White,
                    unfocusedTextColor = Color.White,
                    focusedBorderColor = Indigo500,
                    unfocusedBorderColor = Slate700,
                    focusedLabelColor = Indigo500,
                    unfocusedLabelColor = Slate400
                ),
                shape = RoundedCornerShape(12.dp),
                trailingIcon = {
                    Icon(Lucide.Hash, contentDescription = null, tint = Slate400)
                }
            )
        }
        
        Spacer(modifier = Modifier.height(32.dp))
        
        Button(
            onClick = { 
                val tokens = maxTokensText.toLongOrNull() ?: 0L
                onUpdateMaxTokens(tokens)
            },
            modifier = Modifier
                .fillMaxWidth()
                .height(50.dp),
            colors = ButtonDefaults.buttonColors(containerColor = Indigo500),
            shape = RoundedCornerShape(12.dp),
            enabled = !isUpdating
        ) {
            if (isUpdating) {
                CircularProgressIndicator(modifier = Modifier.size(20.dp), color = Color.White, strokeWidth = 2.dp)
            } else {
                Icon(Lucide.Save, contentDescription = null, modifier = Modifier.size(18.dp))
                Spacer(modifier = Modifier.width(8.dp))
                Text("Save Changes", fontWeight = FontWeight.Bold)
            }
        }
    }
}
