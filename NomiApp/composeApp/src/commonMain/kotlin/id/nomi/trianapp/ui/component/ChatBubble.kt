package id.nomi.trianapp.ui.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.clickable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.composables.icons.lucide.Lucide
import com.composables.icons.lucide.Brain
import com.composables.icons.lucide.ChevronDown
import com.composables.icons.lucide.ChevronRight
import id.nomi.trianapp.ui.*
import id.nomi.trianapp.util.MarkdownBlock
import id.nomi.trianapp.util.MarkdownParser
import id.nomi.trianapp.util.formatTokenCount
import androidx.compose.ui.tooling.preview.Preview

@Composable
fun ChatBubble(
    displayName: String,
    content: String,
    role: String,
    showAvatar: Boolean,
    totalTokens: Long = 0,
    thought: String? = null,
    timestamp: String = "12:00 PM",
    onShowThought:()-> Unit
) {
    val isFromUser = role == "user"
    var isThoughtExpanded by remember { mutableStateOf(false) }

    if(showAvatar){
        Spacer(modifier = Modifier.height(16.dp))
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp, horizontal = 12.dp),
        verticalAlignment = Alignment.Top
    ) {
        // Avatar Placeholder (Slack style)
        if (showAvatar) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(CircleShape)
                    .background(if (isFromUser) Indigo500 else Slate800),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = if(role != "user") "N" else displayName.take(1).uppercase(),
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 16.sp
                )
            }
        } else {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(CircleShape),
                contentAlignment = Alignment.Center
            ) {}
        }
        Spacer(modifier = Modifier.width(12.dp))

        Column(modifier = Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = if(role != "user") "Nomi" else displayName,
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
                if (!isFromUser && totalTokens > 0) {
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = "·",
                        color = Slate400,
                        fontSize = 12.sp
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = "${formatTokenCount(totalTokens)} tokens",
                        style = MaterialTheme.typography.labelSmall.copy(
                            color = Indigo400,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Medium
                        )
                    )
                }
            }

            if (!thought.isNullOrBlank()) {
                Spacer(modifier = Modifier.height(4.dp))
                Surface(
                    color = Slate900.copy(alpha = 0.5f),
                    shape = RoundedCornerShape(8.dp),
                    modifier = Modifier
                        .clip(RoundedCornerShape(8.dp))
                        .clickable {
                            isThoughtExpanded = !isThoughtExpanded
                            if(isThoughtExpanded){
                                onShowThought()
                            }
                        }
                ) {
                    Column(modifier = Modifier.padding(8.dp)) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                Lucide.Brain,
                                contentDescription = null,
                                tint = Indigo500,
                                modifier = Modifier.size(14.dp)
                            )
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(
                                "Deep Thought",
                                color = Slate400,
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Medium
                            )
                            Spacer(modifier = Modifier.weight(1f))
                            Icon(
                                imageVector = if (isThoughtExpanded) Lucide.ChevronDown else Lucide.ChevronRight,
                                contentDescription = null,
                                tint = Slate400,
                                modifier = Modifier.size(14.dp)
                            )
                        }
                        if (isThoughtExpanded) {
                            Spacer(modifier = Modifier.height(6.dp))
                            Text(
                                text = thought,
                                style = MaterialTheme.typography.bodySmall.copy(
                                    color = Slate400,
                                    fontSize = 12.sp,
                                    lineHeight = 18.sp,
                                    fontWeight = FontWeight.Light
                                )
                            )
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(2.dp))

            // Refactored Content rendering with Markdown support
            val blocks = remember(content) { MarkdownParser.parse(content) }

            Column(modifier = Modifier.fillMaxWidth()) {
                blocks.forEach { block ->
                    when (block) {
                        is MarkdownBlock.Text -> {
                            Text(
                                text = block.annotatedString,
                                style = MaterialTheme.typography.bodyMedium.copy(
                                    fontSize = 15.sp,
                                    lineHeight = 22.sp,
                                    color = Slate300
                                ),
                                modifier = Modifier.padding(vertical = 1.dp)
                            )
                        }

                        is MarkdownBlock.Code -> {
                            CodeBlockComponent(
                                code = block.rawCode,
                                language = block.language,
                                modifier = Modifier.padding(vertical = 1.dp)
                            )
                        }
                    }
                }
            }
        }
    }
}

@Preview
@Composable
fun ChatBubbleUserPreview() {
    NomiTheme {
        Box(modifier = Modifier.background(Slate950)) {
            ChatBubble(
                content = "This is a **bold** message from the user with `inline code`.",
                role = "user",
                showAvatar = true,
                displayName = "Trian",
                onShowThought = {}
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
                content = """
                    Here is some code:
                    ```kotlin
                    fun main() {
                        println("Hello World")
                    }
                    ```
                """.trimIndent(),
                role = "assistant",
                showAvatar = true,
                totalTokens = 1250,
                thought = "I need to provide a helpful code example.",
                displayName = "Trian",
                onShowThought = {}
            )
        }
    }
}


@Preview
@Composable
fun ChatBubbleNomiPreview2() {
    NomiTheme {
        Box(modifier = Modifier.background(Slate950)) {
            ChatBubble(
                content = "This is a response from Nomi.",
                role = "assistant",
                showAvatar = false,
                displayName = "Trian",
                onShowThought = {}
            )
        }
    }
}
