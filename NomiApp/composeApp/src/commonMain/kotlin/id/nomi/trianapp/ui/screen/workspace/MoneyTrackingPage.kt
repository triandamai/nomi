package id.nomi.trianapp.ui.screen.workspace

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.composables.icons.lucide.*
import id.nomi.trianapp.data.model.MoneyTransactionDto
import id.nomi.trianapp.data.model.TransactionItemDto
import id.nomi.trianapp.ui.*
import id.nomi.trianapp.util.formatAmount
import id.nomi.trianapp.util.formatTokenCount
import org.koin.compose.viewmodel.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MoneyTrackingPage(
    viewModel: MoneyTrackingViewModel = koinViewModel()
) {
    val state by viewModel.state.collectAsState()
    val selectedTransaction by viewModel.selectedTransaction.collectAsState()
    val listState = rememberLazyListState()
    
    val sheetState = rememberModalBottomSheetState()
    var showBottomSheet by remember { mutableStateOf(false) }

    LaunchedEffect(selectedTransaction) {
        if (selectedTransaction != null) {
            showBottomSheet = true
        }
    }

    // Pagination logic
    val shouldLoadMore = remember {
        derivedStateOf {
            val layoutInfo = listState.layoutInfo
            val totalItemsNumber = layoutInfo.totalItemsCount
            val lastVisibleItemIndex = (layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0) + 1
            lastVisibleItemIndex > (totalItemsNumber - 2)
        }
    }

    LaunchedEffect(shouldLoadMore.value) {
        if (shouldLoadMore.value && state is MoneyTrackingState.Success && (state as MoneyTrackingState.Success).hasMore) {
            viewModel.fetchHistory()
        }
    }

    Box(modifier = Modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(24.dp)
        ) {
            Text(
                "Money Tracking",
                color = Color.White,
                fontSize = 24.sp,
                fontWeight = FontWeight.Bold
            )
            Spacer(modifier = Modifier.height(20.dp))

            when (val s = state) {
                is MoneyTrackingState.Loading -> {
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        CircularProgressIndicator(color = Indigo500)
                    }
                }
                is MoneyTrackingState.Error -> {
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Text(s.message, color = Color.Red)
                    }
                }
                is MoneyTrackingState.Success -> {
                    if (s.items.isEmpty()) {
                        EmptyMoneyState()
                    } else {
                        LazyColumn(
                            state = listState,
                            verticalArrangement = Arrangement.spacedBy(12.dp),
                            contentPadding = PaddingValues(bottom = 24.dp)
                        ) {
                            items(s.items) { transaction ->
                                MoneyTransactionCard(transaction) {
                                    viewModel.selectTransaction(transaction)
                                }
                            }
                            if (s.hasMore) {
                                item {
                                    Box(modifier = Modifier.fillMaxWidth().padding(16.dp), contentAlignment = Alignment.Center) {
                                        CircularProgressIndicator(modifier = Modifier.size(24.dp), color = Indigo500)
                                    }
                                }
                            }
                        }
                    }
                }
                else -> {}
            }
        }

        // Bottom Sheet for Detail
        if (showBottomSheet && selectedTransaction != null) {
            ModalBottomSheet(
                onDismissRequest = {
                    showBottomSheet = false
                    viewModel.selectTransaction(null)
                },
                sheetState = sheetState,
                containerColor = Slate950,
            ) {
                TransactionDetailContent(
                    transaction = selectedTransaction!!,
                    onClose = {
                        showBottomSheet = false
                        viewModel.selectTransaction(null)
                    }
                )
            }
        }
    }
}

@Composable
fun MoneyTransactionCard(transaction: MoneyTransactionDto, onClick: () -> Unit) {
    Surface(
        color = Slate900,
        shape = RoundedCornerShape(16.dp),
        modifier = Modifier.fillMaxWidth().clickable { onClick() }
    ) {
        Row(
            modifier = Modifier.padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(44.dp)
                    .clip(CircleShape)
                    .background(Slate800),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = when(transaction.category.lowercase()) {
                        "food & beverage" -> Lucide.Utensils
                        "entertainment" -> Lucide.Ticket
                        "shopping" -> Lucide.ShoppingBag
                        "transportation" -> Lucide.Bus
                        else -> Lucide.Wallet
                    },
                    contentDescription = null,
                    tint = Color.White,
                    modifier = Modifier.size(20.dp)
                )
            }
            Spacer(modifier = Modifier.width(16.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    transaction.merchantName,
                    color = Slate100,
                    fontSize = 16.sp,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    transaction.category,
                    color = Slate400,
                    fontSize = 13.sp
                )
            }
            Column(horizontalAlignment = Alignment.End) {
                Text(
                    "IDR ${formatAmount(transaction.totalAmount?.toLong() ?:0)}",
                    color = Color.White,
                    fontSize = 16.sp,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    transaction.createdAt.split("T").firstOrNull() ?: "",
                    color = Slate400,
                    fontSize = 12.sp
                )
            }
        }
    }
}

@Composable
fun TransactionDetailContent(transaction: MoneyTransactionDto, onClose: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 24.dp)
            .padding(bottom = 32.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Box(
            modifier = Modifier
                .size(64.dp)
                .clip(CircleShape)
                .background(Slate800),
            contentAlignment = Alignment.Center
        ) {
            Icon(Lucide.Receipt, contentDescription = null, tint = Indigo500, modifier = Modifier.size(32.dp))
        }
        
        Spacer(modifier = Modifier.height(16.dp))
        
        Text(
            transaction.merchantName,
            color = Color.White,
            fontSize = 20.sp,
            fontWeight = FontWeight.Bold
        )
        Text(
            transaction.category,
            color = Slate400,
            fontSize = 14.sp
        )
        
        Spacer(modifier = Modifier.height(24.dp))
        
        Text(
            "IDR ${formatAmount(transaction.totalAmount?.toLong() ?:0)}",
            color = Color.White,
            fontSize = 32.sp,
            fontWeight = FontWeight.ExtraBold
        )
        
        Spacer(modifier = Modifier.height(24.dp))
        HorizontalDivider(color = Slate800)
        Spacer(modifier = Modifier.height(24.dp))
        
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            DetailItem("Date", transaction.createdAt.split("T").firstOrNull() ?: "")
            DetailItem("Time", transaction.createdAt.split("T").lastOrNull()?.take(5) ?: "")
        }
        
        if (transaction.items.isNotEmpty()) {
            Spacer(modifier = Modifier.height(24.dp))
            Text(
                "Items Purchased",
                modifier = Modifier.fillMaxWidth(),
                color = Slate300,
                fontSize = 14.sp,
                fontWeight = FontWeight.SemiBold
            )
            Spacer(modifier = Modifier.height(12.dp))
            LazyColumn(modifier = Modifier.heightIn(max = 200.dp)) {
                items(transaction.items) { item ->
                    PurchasedItemRow(item)
                }
            }
        }
        
        Spacer(modifier = Modifier.height(32.dp))
        Button(
            onClick = onClose,
            modifier = Modifier.fillMaxWidth().height(50.dp),
            colors = ButtonDefaults.buttonColors(containerColor = Slate800),
            shape = RoundedCornerShape(12.dp)
        ) {
            Text("Close", color = Color.White)
        }
    }
}

@Composable
fun DetailItem(label: String, value: String) {
    Column {
        Text(label, color = Slate400, fontSize = 12.sp)
        Text(value, color = Color.White, fontSize = 15.sp, fontWeight = FontWeight.Medium)
    }
}

@Composable
fun PurchasedItemRow(item: TransactionItemDto) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 6.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            "${item.quantity}x ${item.name}",
            color = Slate100,
            fontSize = 14.sp,
            modifier = Modifier.weight(1f),
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
        Text(
            "IDR ${formatAmount(item.totalAmount.toLong())}",
            color = Color.White,
            fontSize = 14.sp,
            fontWeight = FontWeight.SemiBold
        )
    }
}

@Composable
fun EmptyMoneyState() {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Icon(Lucide.Wallet, contentDescription = null, tint = Slate700, modifier = Modifier.size(64.dp))
        Spacer(modifier = Modifier.height(16.dp))
        Text("No transactions recorded", color = Slate400, fontSize = 16.sp)
    }
}
