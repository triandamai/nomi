package id.nomi.trianapp.ui.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import id.nomi.trianapp.ui.*
import id.nomi.trianapp.util.MarkdownBlock
import id.nomi.trianapp.util.MarkdownParser
import androidx.compose.ui.tooling.preview.Preview

@Composable
fun ChatBubble(
    content: String,
    isFromUser: Boolean,
    showAvatar: Boolean,
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
        if (showAvatar) {
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
                isFromUser = true,
                showAvatar = true
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
                isFromUser = false,
                showAvatar = true
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
                isFromUser = true,
                showAvatar = false
            )
        }
    }
}
