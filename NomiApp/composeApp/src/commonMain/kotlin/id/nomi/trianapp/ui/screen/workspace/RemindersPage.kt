package id.nomi.trianapp.ui.screen.workspace

import androidx.compose.foundation.background
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
import id.nomi.trianapp.data.model.ReminderDto
import id.nomi.trianapp.ui.*
import org.koin.compose.viewmodel.koinViewModel

@Composable
fun RemindersPage(
    viewModel: RemindersViewModel = koinViewModel()
) {
    val state by viewModel.state.collectAsState()

    LaunchedEffect(Unit) {
        viewModel.fetchReminders()
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp)
    ) {
        Text(
            "Reminders",
            color = Color.White,
            fontSize = 24.sp,
            fontWeight = FontWeight.Bold
        )
        Spacer(modifier = Modifier.height(20.dp))

        when (val s = state) {
            is RemindersState.Loading -> {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator(color = Indigo500)
                }
            }
            is RemindersState.Error -> {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text(s.message, color = Color.Red)
                }
            }
            is RemindersState.Success -> {
                if (s.reminders.isEmpty()) {
                    EmptyRemindersState()
                } else {
                    LazyColumn(
                        verticalArrangement = Arrangement.spacedBy(16.dp),
                        contentPadding = PaddingValues(bottom = 24.dp)
                    ) {
                        items(s.reminders) { reminder ->
                            ReminderCard(reminder)
                        }
                    }
                }
            }
            else -> {}
        }
    }
}

@Composable
fun ReminderCard(reminder: ReminderDto) {
    Surface(
        color = Slate900,
        shape = RoundedCornerShape(16.dp),
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier.padding(20.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Status Icon
                Box(
                    modifier = Modifier
                        .size(40.dp)
                        .clip(CircleShape)
                        .background(if (reminder.status == "pending") Indigo500.copy(alpha = 0.1f) else Emerald500.copy(alpha = 0.1f)),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        imageVector = if (reminder.status == "pending") Lucide.Clock else Lucide.Check,
                        contentDescription = null,
                        tint = if (reminder.status == "pending") Indigo500 else Emerald500,
                        modifier = Modifier.size(20.dp)
                    )
                }
                
                Spacer(modifier = Modifier.width(16.dp))
                
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = reminder.content,
                        color = Slate100,
                        fontSize = 16.sp,
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            Lucide.Calendar,
                            contentDescription = null,
                            tint = Slate400,
                            modifier = Modifier.size(12.dp)
                        )
                        Spacer(modifier = Modifier.width(4.dp))
                        Text(
                            text = reminder.dueAt.replace("T", " ").replace("Z", ""),
                            color = Slate400,
                            fontSize = 12.sp
                        )
                    }
                }
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            Divider(color = Slate800, thickness = 1.dp)
            Spacer(modifier = Modifier.height(16.dp))
            
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Topic / Conversation Badge
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .clip(RoundedCornerShape(8.dp))
                        .background(Slate800)
                        .padding(horizontal = 8.dp, vertical = 4.dp)
                ) {
                    Icon(
                        Lucide.Hash,
                        contentDescription = null,
                        tint = Slate400,
                        modifier = Modifier.size(12.dp)
                    )
                    Spacer(modifier = Modifier.width(4.dp))
                    Text(
                        text = reminder.conversationTitle,
                        color = Slate400,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Medium
                    )
                }
                
                // Frequency Badge
                Row(
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        Lucide.Repeat,
                        contentDescription = null,
                        tint = Indigo400,
                        modifier = Modifier.size(14.dp)
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text(
                        text = reminder.frequency.replaceFirstChar { it.uppercase() },
                        color = Indigo400,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }
        }
    }
}

@Composable
fun EmptyRemindersState() {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(
            Lucide.BellOff,
            contentDescription = null,
            tint = Slate700,
            modifier = Modifier.size(64.dp)
        )
        Spacer(modifier = Modifier.height(16.dp))
        Text(
            "No reminders found",
            color = Slate400,
            fontSize = 16.sp
        )
    }
}
