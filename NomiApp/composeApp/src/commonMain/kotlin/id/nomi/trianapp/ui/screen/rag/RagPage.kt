package id.nomi.trianapp.ui.screen.rag

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import id.nomi.trianapp.ui.Slate950
import androidx.compose.ui.tooling.preview.Preview
import id.nomi.trianapp.ui.NomiTheme

@Composable
fun RagPage() {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Slate950),
        contentAlignment = Alignment.Center
    ) {
        Text("RAG Page", color = Color.White, style = MaterialTheme.typography.headlineMedium)
    }
}

@Preview
@Composable
fun RagPagePreview() {
    NomiTheme {
        RagPage()
    }
}
