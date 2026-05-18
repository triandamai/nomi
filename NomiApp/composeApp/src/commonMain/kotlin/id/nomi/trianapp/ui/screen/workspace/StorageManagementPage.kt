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
import coil3.compose.AsyncImage
import com.composables.icons.lucide.*
import id.nomi.trianapp.data.model.StorageItemDto
import id.nomi.trianapp.data.remote.baseUrl
import id.nomi.trianapp.formatBytes
import id.nomi.trianapp.ui.*
import id.nomi.trianapp.ui.screen.admin.DetailItem
import id.nomi.trianapp.ui.screen.admin.DetailSection
import id.nomi.trianapp.util.formatAmount
import org.koin.compose.viewmodel.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun StorageManagementPage(
    viewModel: StorageManagementViewModel = koinViewModel()
) {
    val uiState by viewModel.uiState.collectAsState()
    var selectedFile by remember { mutableStateOf<StorageItemDto?>(null) }
    val sheetState = rememberModalBottomSheetState()
    var showBottomSheet by remember { mutableStateOf(false) }

    LaunchedEffect(selectedFile) {
        if (selectedFile != null) {
            showBottomSheet = true
        }
    }

    Box(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(24.dp)
        ) {
            // Header with Breadcrumbs/Back
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (uiState.currentPath.isNotEmpty()) {
                    IconButton(onClick = { viewModel.navigateBack() }) {
                        Icon(Lucide.ChevronLeft, contentDescription = "Back", tint = Color.White)
                    }
                }
                Column {
                    Text(
                        "Storage Management",
                        color = Color.White,
                        fontSize = 24.sp,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        if (uiState.currentPath.isEmpty()) "Root /" else uiState.currentPath,
                        color = Slate400,
                        fontSize = 14.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
            
            Spacer(modifier = Modifier.height(24.dp))

            if (uiState.isLoading && uiState.items.isEmpty()) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator(color = Indigo500)
                }
            } else if (uiState.error != null && uiState.items.isEmpty()) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text(uiState.error!!, color = Color.Red)
                }
            } else {
                LazyColumn(
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.fillMaxSize()
                ) {
                    items(uiState.items) { item ->
                        StorageItemRow(
                            item = item,
                            onClick = {
                                if (item.type == "folder" || item.type == "bucket") {
                                    viewModel.explore(item.fullPath)
                                } else {
                                    selectedFile = item
                                }
                            }
                        )
                    }
                }
            }
        }

        if (showBottomSheet && selectedFile != null) {
            ModalBottomSheet(
                onDismissRequest = {
                    showBottomSheet = false
                    selectedFile = null
                },
                sheetState = sheetState,
                containerColor = Slate950
            ) {
                StorageFileDetailContent(
                    item = selectedFile!!,
                    onDelete = {
                        viewModel.deleteFile(selectedFile!!)
                        showBottomSheet = false
                        selectedFile = null
                    }
                )
            }
        }
        
        if (uiState.isDeleting) {
            Box(modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.5f)), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(color = Indigo500)
            }
        }
    }
}

@Composable
fun StorageItemRow(
    item: StorageItemDto,
    onClick: () -> Unit
) {
    Surface(
        color = Slate900,
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
    ) {
        Row(
            modifier = Modifier.padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(Slate800),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = when(item.type) {
                        "bucket" -> Lucide.Database
                        "folder" -> Lucide.Folder
                        else -> Lucide.File
                    },
                    contentDescription = null,
                    tint = when(item.type) {
                        "bucket" -> Indigo500
                        "folder" -> Color(0xFFFFD700)
                        else -> Color.White
                    },
                    modifier = Modifier.size(20.dp)
                )
            }
            Spacer(modifier = Modifier.width(16.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    item.name,
                    color = Color.White,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Medium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                if (item.type == "file") {
                    Text(
                        "${item.size?.let { formatBytes(it) } ?: "0 B"} • ${item.contentType ?: "unknown"}",
                        color = Slate400,
                        fontSize = 12.sp
                    )
                } else {
                    Text(
                        item.type.replaceFirstChar { it.uppercase() },
                        color = Slate400,
                        fontSize = 12.sp
                    )
                }
            }
            Icon(Lucide.ChevronRight, contentDescription = null, tint = Slate700, modifier = Modifier.size(16.dp))
        }
    }
}

@Composable
fun StorageFileDetailContent(
    item: StorageItemDto,
    onDelete: () -> Unit
) {
    val isImage = item.contentType?.startsWith("image/") == true
    val previewUrl = "${baseUrl}/api/v1/admin/storage/file?path=${item.fullPath.replace("/", "%2F")}"

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 24.dp)
            .padding(bottom = 32.dp)
    ) {
        Text(
            "File Details",
            color = Color.White,
            fontSize = 20.sp,
            fontWeight = FontWeight.Bold
        )
        Spacer(modifier = Modifier.height(24.dp))

        if (isImage) {
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(240.dp),
                color = Slate900,
                shape = RoundedCornerShape(12.dp)
            ) {
                AsyncImage(
                    model = previewUrl,
                    contentDescription = "Preview",
                    modifier = Modifier.fillMaxSize()
                )
            }
            Spacer(modifier = Modifier.height(24.dp))
        }

        HorizontalDivider(color = Slate800)
        Spacer(modifier = Modifier.height(24.dp))

        DetailSection("File Information") {
            DetailItem("Name", item.name)
            DetailItem("Path", item.fullPath)
            DetailItem("Size", item.size?.let { formatBytes(it) } ?: "0 B")
            DetailItem("Content Type", item.contentType ?: "unknown")
            DetailItem("Last Modified", item.updatedAt ?: "-")
        }

        Spacer(modifier = Modifier.height(32.dp))
        
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            OutlinedButton(
                onClick = { /* Download logic */ },
                modifier = Modifier.weight(1f).height(50.dp),
                border = borderStroke(1.dp, Slate700),
                shape = RoundedCornerShape(12.dp),
                colors = ButtonDefaults.outlinedButtonColors(contentColor = Color.White)
            ) {
                Icon(Lucide.Download, contentDescription = null, modifier = Modifier.size(18.dp))
                Spacer(modifier = Modifier.width(8.dp))
                Text("Download")
            }
            
            Button(
                onClick = onDelete,
                modifier = Modifier.weight(1f).height(50.dp),
                colors = ButtonDefaults.buttonColors(containerColor = Color.Red.copy(alpha = 0.8f)),
                shape = RoundedCornerShape(12.dp)
            ) {
                Icon(Lucide.Trash2, contentDescription = null, modifier = Modifier.size(18.dp))
                Spacer(modifier = Modifier.width(8.dp))
                Text("Delete")
            }
        }
    }
}

private fun borderStroke(width: androidx.compose.ui.unit.Dp, color: Color) = androidx.compose.foundation.BorderStroke(width, color)
