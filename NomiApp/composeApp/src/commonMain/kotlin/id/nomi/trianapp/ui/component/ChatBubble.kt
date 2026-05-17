package id.nomi.trianapp.ui.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import id.nomi.trianapp.ui.*
import androidx.compose.ui.tooling.preview.Preview

@Composable
fun ChatBubble(
    content: String,
    isFromUser: Boolean,
    senderName: String = if (isFromUser) "You" else "Nomi",
    timestamp: String = "12:00 PM"
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp, horizontal = 12.dp),
        verticalAlignment = Alignment.Top
    ) {
        // Avatar Placeholder (Slack style)
        Box(
            modifier = Modifier
                .size(40.dp)
                .clip(CircleShape)
                .background(if (isFromUser) Indigo500 else Slate800),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = senderName.take(1).uppercase(),
                color = Color.White,
                fontWeight = FontWeight.Bold,
                fontSize = 16.sp
            )
        }

        Spacer(modifier = Modifier.width(12.dp))

        Column(modifier = Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = senderName,
                    style = MaterialTheme.typography.bodyLarge.copy(
                        fontWeight = FontWeight.Bold,
                        fontSize = 15.sp,
                        color = Slate100
                    )
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text(
                    text = timestamp,
                    style = MaterialTheme.typography.labelSmall.copy(
                        color = Slate400,
                        fontSize = 12.sp
                    )
                )
            }
            
            Spacer(modifier = Modifier.height(2.dp))
            
            Text(
                text = content,
                style = MaterialTheme.typography.bodyMedium.copy(
                    fontSize = 15.sp,
                    lineHeight = 22.sp,
                    color = Slate300
                )
            )
        }
    }
}

@Preview
@Composable
fun ChatBubbleUserPreview() {
    NomiTheme {
        Box(modifier = Modifier.background(Slate950)) {
            ChatBubble(
                content = "This is a message from the user.",
                isFromUser = true
            )
        }
    }
}

@Preview
@Composable
fun ChatBubbleNomiPreview() {
    NomiTheme {
        Box(modifier = Modifier.background(Slate950)) {
            ChatBubble(
                content = "This is a response from Nomi.",
                isFromUser = false
            )
        }
    }
}
