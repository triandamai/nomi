package id.nomi.trianapp.ui.component

import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.composables.icons.lucide.Copy
import com.composables.icons.lucide.ChevronDown
import com.composables.icons.lucide.ChevronRight
import com.composables.icons.lucide.Lucide

@Composable
fun CodeBlockComponent(
    code: String,
    language: String,
    modifier: Modifier = Modifier
) {
    var isCollapsed by remember { mutableStateOf(false) }

    Surface(
        color = Color(0xFF020617), // Slate-950
        shape = RoundedCornerShape(8.dp),
        modifier = modifier.fillMaxWidth()
    ) {
        Column {
            // Header Row
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(Color(0xFF1e293b).copy(alpha = 0.5f)) // Translucent Slate-800
                    .padding(horizontal = 12.dp, vertical = 6.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = language.lowercase(),
                    color = Color(0xFF94a3b8), // Slate-400
                    fontSize = 12.sp,
                    fontFamily = FontFamily.Monospace
                )
                
                Row(verticalAlignment = Alignment.CenterVertically) {
                    IconButton(
                        onClick = { /* Copy logic would go here */ },
                        modifier = Modifier.size(24.dp)
                    ) {
                        Icon(
                            imageVector = Lucide.Copy,
                            contentDescription = "Copy",
                            tint = Color(0xFF94a3b8), // Slate-400
                            modifier = Modifier.size(16.dp)
                        )
                    }

                    Spacer(modifier = Modifier.width(8.dp))

                    IconButton(
                        onClick = { isCollapsed = !isCollapsed },
                        modifier = Modifier.size(24.dp)
                    ) {
                        Icon(
                            imageVector = if (isCollapsed) Lucide.ChevronRight else Lucide.ChevronDown,
                            contentDescription = if (isCollapsed) "Expand" else "Collapse",
                            tint = Color(0xFF94a3b8), // Slate-400
                            modifier = Modifier.size(16.dp)
                        )
                    }
                }
            }
            
            // Code Content
            if (!isCollapsed) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .horizontalScroll(rememberScrollState())
                        .padding(16.dp)
                ) {
                    Text(
                        text = code,
                        color = Color(0xFFe2e8f0), // Slate-200
                        fontSize = 13.sp,
                        fontFamily = FontFamily.Monospace,
                        softWrap = false
                    )
                }
            }
        }
    }
}
