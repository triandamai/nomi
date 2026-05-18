package id.nomi.trianapp.ui.screen.rag

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
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
import id.nomi.trianapp.ui.Slate950
import com.multiplatform.webview.web.WebView
import com.multiplatform.webview.web.rememberWebViewStateWithHTMLData
import com.multiplatform.webview.web.rememberWebViewNavigator
import com.multiplatform.webview.jsbridge.rememberWebViewJsBridge
import com.multiplatform.webview.jsbridge.IJsMessageHandler
import com.multiplatform.webview.jsbridge.JsMessage
import org.koin.compose.viewmodel.koinViewModel
import org.jetbrains.compose.resources.ExperimentalResourceApi
import arta.composeapp.generated.resources.Res
import com.composables.icons.lucide.Lucide
import com.composables.icons.lucide.MessageSquare
import com.multiplatform.webview.web.WebViewNavigator
import id.nomi.trianapp.ui.Emerald500
import id.nomi.trianapp.ui.Slate400
import id.nomi.trianapp.ui.Slate800
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

@OptIn(ExperimentalResourceApi::class, ExperimentalMaterial3Api::class)
@Composable
fun RagPage(
    viewModel: RagViewModel = koinViewModel(),
    onNavigationClick: () -> Unit
) {
    val graphData by viewModel.graphData.collectAsState()
    val isLoadingDetails by viewModel.isLoadingDetails.collectAsState()
    val selectedNodeDetails by viewModel.selectedNodeDetails.collectAsState()

    val navigator = rememberWebViewNavigator()
    val jsBridge = rememberWebViewJsBridge()
    var htmlContent by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        try {
            htmlContent = Res.readBytes("files/rag_graph_3d.html").decodeToString()
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    // Register JS message handler via JsBridge
    LaunchedEffect(jsBridge) {
        jsBridge.register(object : IJsMessageHandler {
            override fun handle(
                message: JsMessage,
                navigator: WebViewNavigator?,
                callback: (String) -> Unit
            ) {
                try {
                    val json = Json.parseToJsonElement(message.params).jsonObject
                    if (json["event"]?.jsonPrimitive?.content == "node_clicked") {
                        val nodeId = json["id"]?.jsonPrimitive?.content
                        if (nodeId != null) {
                            viewModel.loadNodeDetails(nodeId)
                        }
                    }
                } catch (e: Exception) {
                    e.printStackTrace()
                }
            }


            override fun methodName(): String = "postMessage"
        })
    }

    val state = htmlContent?.let { content ->
        rememberWebViewStateWithHTMLData(
            data = content,
            baseUrl = null
        ).apply {
            webSettings.isJavaScriptEnabled = true
        }
    }

    LaunchedEffect(graphData, state?.loadingState) {
        if (graphData.isNotEmpty() && state?.loadingState is com.multiplatform.webview.web.LoadingState.Finished) {
            navigator.evaluateJavaScript("window.update3DGraph(`${graphData}`)")
        }
    }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
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
                                    "Conversation",
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
                        titleContentColor = Color.White,
                    ),
                    navigationIcon = {
                        IconButton(onClick = onNavigationClick) {
                            Icon(
                                imageVector = Lucide.MessageSquare,
                                contentDescription = null,
                                tint = Color.White
                            )
                        }
                    }
                )
                Divider(color = Slate800, thickness = 0.5.dp)
            }
        },
        containerColor = Slate950
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            state?.let {
                WebView(
                    state = it,
                    modifier = Modifier.fillMaxSize(),
                    navigator = navigator,
                    webViewJsBridge = jsBridge
                )
            }

            if (isLoadingDetails) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(Color.Black.copy(alpha = 0.3f)),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator(
                        color = Color(0xFF10b981) // Emerald-500
                    )
                }
            }
        }
    }

    if (selectedNodeDetails != null) {
        ModalBottomSheet(
            onDismissRequest = { viewModel.dismissNodeDetails() },
            containerColor = Color(0xFF0f172a), // Slate-900
            shape = RoundedCornerShape(
                topStart = 16.dp,
                topEnd = 16.dp,
                bottomStart = 0.dp,
                bottomEnd = 0.dp
            ),
            dragHandle = { BottomSheetDefaults.DragHandle(color = Color.Gray) }
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(start = 24.dp, end = 24.dp, bottom = 48.dp)
            ) {
                Text(
                    text = selectedNodeDetails?.label ?: "",
                    color = Color.White,
                    fontSize = 20.sp,
                    fontWeight = FontWeight.Bold
                )

                Spacer(modifier = Modifier.height(8.dp))

                Surface(
                    color = Color(0xFF1e293b), // Slate-800
                    shape = RoundedCornerShape(8.dp)
                ) {
                    Text(
                        text = selectedNodeDetails?.nodeType?.uppercase() ?: "",
                        color = Color(0xFF38bdf8), // Cyan-400
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Medium,
                        modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp)
                    )
                }

                Spacer(modifier = Modifier.height(16.dp))

                Card(
                    colors = CardDefaults.cardColors(containerColor = Color(0xFF1e293b)),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text(
                            text = "Metadata Information",
                            color = Color.Gray,
                            fontSize = 12.sp,
                            fontWeight = FontWeight.SemiBold
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "This node represents a ${selectedNodeDetails?.nodeType} extracted from the RAG knowledge graph. ID: ${selectedNodeDetails?.id}",
                            color = Color.LightGray,
                            fontSize = 14.sp
                        )
                    }
                }
            }
        }
    }
}
