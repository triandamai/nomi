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
import com.composables.icons.lucide.*
import id.nomi.trianapp.data.model.SoulHistoryDto
import id.nomi.trianapp.ui.*
import id.nomi.trianapp.ui.screen.admin.DetailSection
import org.koin.compose.viewmodel.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SoulTimelinePage(
    viewModel: SoulTimelineViewModel = koinViewModel()
) {
    val uiState by viewModel.uiState.collectAsState()
    var selectedSoul by remember { mutableStateOf<SoulHistoryDto?>(null) }
    val sheetState = rememberModalBottomSheetState()
    var showBottomSheet by remember { mutableStateOf(false) }

    LaunchedEffect(selectedSoul) {
        if (selectedSoul != null) {
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
                "Soul Timeline",
                color = Color.White,
                fontSize = 24.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                "History of soul updates and personality shifts",
                color = Slate400,
                fontSize = 14.sp
            )
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
                    verticalArrangement = Arrangement.spacedBy(16.dp),
                    modifier = Modifier.fillMaxSize()
                ) {
                    items(uiState.items) { item ->
                        SoulTimelineItem(
                            item = item,
                            isSelected = selectedSoul?.id == item.id,
                            onClick = { selectedSoul = item },
                            onRestore = { viewModel.restoreSoul(item.id) }
                        )
                    }
                }
            }
        }

        // Bottom Sheet for Detail
        if (showBottomSheet && selectedSoul != null) {
            ModalBottomSheet(
                onDismissRequest = {
                    showBottomSheet = false
                    selectedSoul = null
                },
                sheetState = sheetState,
                containerColor = Slate950,
               // dragHandleColor = Slate400
            ) {
                SoulDetailContent(
                    soul = selectedSoul!!,
                    onRestore = {
                        viewModel.restoreSoul(selectedSoul!!.id)
                        showBottomSheet = false
                        selectedSoul = null
                    }
                )
            }
        }

        if (uiState.isRestoring) {
            Box(
                modifier = Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.5f)),
                contentAlignment = Alignment.Center
            ) {
                CircularProgressIndicator(color = Indigo500)
            }
        }
    }
}

@Composable
fun SoulDetailContent(
    soul: SoulHistoryDto,
    onRestore: () -> Unit
) {
    var isContentExpanded by remember { mutableStateOf(false) }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 24.dp)
            .padding(bottom = 32.dp)
    ) {
        // Header (Non-scrollable)
        Column {
            Text(
                "Soul Version ${soul.version}",
                color = Color.White,
                fontSize = 20.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                soul.createdAt,
                color = Slate400,
                fontSize = 13.sp
            )
        }

        Spacer(modifier = Modifier.height(24.dp))
        HorizontalDivider(color = Slate800)
        Spacer(modifier = Modifier.height(24.dp))

        // Scrollable content
        LazyColumn(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(24.dp)
        ) {
            item {
                DetailSection("Change Reason") {
                    Text(
                        soul.changeReason,
                        color = Color.White,
                        fontSize = 15.sp,
                        lineHeight = 22.sp
                    )
                }
            }

            item {
                Column {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable { isContentExpanded = !isContentExpanded }
                            .padding(vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(
                            "Soul Content",
                            color = Indigo500,
                            fontSize = 14.sp,
                            fontWeight = FontWeight.Bold
                        )
                        Icon(
                            imageVector = if (isContentExpanded) Lucide.ChevronUp else Lucide.ChevronDown,
                            contentDescription = null,
                            tint = Indigo500,
                            modifier = Modifier.size(20.dp)
                        )
                    }

                    AnimatedVisibility(
                        visible = isContentExpanded,
                        enter = expandVertically() + fadeIn(),
                        exit = shrinkVertically() + fadeOut()
                    ) {
                        Column {
                            Spacer(modifier = Modifier.height(12.dp))
                            if (soul.soulContent.isEmpty()) {
                                Text("No content recorded for this version", color = Slate400, fontSize = 14.sp)
                            } else {
                                Surface(
                                    color = Slate900,
                                    shape = RoundedCornerShape(8.dp),
                                    modifier = Modifier.fillMaxWidth()
                                ) {
                                    Box(modifier = Modifier.padding(16.dp)) {
                                        Text(
                                            soul.soulContent,
                                            color = Color.LightGray,
                                            fontSize = 14.sp,
                                            lineHeight = 20.sp
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fixed Restore Button
        Spacer(modifier = Modifier.height(24.dp))
        Button(
            onClick = onRestore,
            modifier = Modifier.fillMaxWidth(),
            colors = ButtonDefaults.buttonColors(containerColor = Indigo500),
            shape = RoundedCornerShape(8.dp),
            contentPadding = PaddingValues(16.dp)
        ) {
            Icon(Lucide.RotateCcw, contentDescription = null, modifier = Modifier.size(18.dp))
            Spacer(modifier = Modifier.width(8.dp))
            Text("Restore to this version", fontWeight = FontWeight.Bold)
        }
    }
}

@Composable
fun SoulTimelineItem(
    item: SoulHistoryDto,
    isSelected: Boolean,
    onClick: () -> Unit,
    onRestore: () -> Unit
) {
    Surface(
        color = if (isSelected) Slate800 else Slate900,
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onClick() }
            .border(
                width = 1.dp,
                color = if (isSelected) Indigo500 else Color.Transparent,
                shape = RoundedCornerShape(12.dp)
            )
    ) {
        Row(
            modifier = Modifier.padding(16.dp),
            verticalAlignment = Alignment.Top
        ) {
            // Version Badge
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(CircleShape)
                    .background(if (isSelected) Indigo500 else Slate800),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "v${item.version}",
                    color = Color.White,
                    fontWeight = FontWeight.Bold,
                    fontSize = 14.sp
                )
            }

            Spacer(modifier = Modifier.width(16.dp))

            Column(modifier = Modifier.weight(1f)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = item.createdAt.split("T").firstOrNull() ?: "",
                        color = Slate400,
                        fontSize = 12.sp
                    )

                    Surface(
                        color = Indigo500.copy(alpha = 0.1f),
                        shape = RoundedCornerShape(4.dp),
                        modifier = Modifier.clickable { onRestore() }
                    ) {
                        Row(
                            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Icon(
                                Lucide.RotateCcw,
                                contentDescription = null,
                                tint = Indigo500,
                                modifier = Modifier.size(12.dp)
                            )
                            Spacer(modifier = Modifier.width(4.dp))
                            Text("Restore", color = Indigo500, fontSize = 11.sp, fontWeight = FontWeight.Bold)
                        }
                    }
                }

                Spacer(modifier = Modifier.height(4.dp))

                Text(
                    item.changeReason,
                    color = Color.White,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Medium,
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}
