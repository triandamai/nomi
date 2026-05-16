package id.nomi.trianapp.ui.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun ChatBubble(
    content: String,
    isFromUser: Boolean
) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = if (isFromUser) Alignment.End else Alignment.Start
    ) {
        Box(
            modifier = Modifier
                .widthIn(max = 280.dp)
                .background(
                    color = if (isFromUser) Color(0xFF1E293B) else Color(0xFF0F172A), // Slate 800 and Slate 900
                    shape = RoundedCornerShape(
                        topStart = 12.dp,
                        topEnd = 12.dp,
                        bottomStart = if (isFromUser) 12.dp else 0.dp,
                        bottomEnd = if (isFromUser) 0.dp else 12.dp
                    )
                )
                .padding(12.dp)
        ) {
            Text(
                text = content,
                color = Color(0xFFE2E8F0), // Slate 200
                fontSize = 14.sp
            )
        }
    }
}
