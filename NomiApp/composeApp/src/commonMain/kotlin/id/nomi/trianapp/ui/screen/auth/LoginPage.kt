package id.nomi.trianapp.ui.screen.auth

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.*
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.composables.icons.lucide.Lucide
import com.composables.icons.lucide.ShieldCheck
import com.composables.icons.lucide.Smartphone
import id.nomi.trianapp.ui.*
import org.koin.compose.viewmodel.koinViewModel

class PairingCodeTransformation : VisualTransformation {
    override fun filter(text: AnnotatedString): TransformedText {
        val trimmed = if (text.text.length >= 6) text.text.substring(0, 6) else text.text
        var out = ""
        for (i in trimmed.indices) {
            out += trimmed[i]
            if (i == 2 && i != trimmed.lastIndex) out += "-"
        }

        val offsetMapping = object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int {
                if (offset <= 2) return offset
                if (offset <= 6) {
                    // Only add the offset for the hyphen if the hyphen actually exists in 'out'
                    return if (out.length > 3) offset + 1 else offset
                }
                return out.length
            }

            override fun transformedToOriginal(offset: Int): Int {
                if (offset <= 3) return offset.coerceAtMost(text.text.length)
                if (offset <= 7) return (offset - 1).coerceAtMost(text.text.length)
                return text.text.length
            }
        }

        return TransformedText(AnnotatedString(out), offsetMapping)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LoginPage(
    onPairingSuccess: () -> Unit = {}
) {
    val viewModel: LoginViewModel = koinViewModel()
    val state by viewModel.loginState.collectAsState()
    var pairingCode by remember { mutableStateOf("") }
    val snackbarHostState = remember { SnackbarHostState() }

    LaunchedEffect(state) {
        if (state is LoginState.Success) {
            onPairingSuccess()
        }
        if (state is LoginState.Error) {
            snackbarHostState.showSnackbar(
                message = (state as LoginState.Error).message,
                actionLabel = "Dismiss",
                duration = SnackbarDuration.Short
            )
        }
    }

    Scaffold(
        snackbarHost = {
            SnackbarHost(snackbarHostState) { data ->
                Snackbar(
                    snackbarData = data,
                    containerColor = Slate900,
                    contentColor = Slate100,
                    actionColor = Indigo400,
                    shape = RoundedCornerShape(12.dp)
                )
            }
        },
        containerColor = Slate950
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding),
            contentAlignment = Alignment.Center
        ) {
            Column(
                modifier = Modifier
                    .widthIn(max = 400.dp)
                    .padding(24.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                // Logo / Icon Section
                Box(
                    modifier = Modifier
                        .size(64.dp)
                        .background(Indigo500.copy(alpha = 0.1f), RoundedCornerShape(16.dp)),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        imageVector = Lucide.ShieldCheck,
                        contentDescription = null,
                        tint = Indigo500,
                        modifier = Modifier.size(32.dp)
                    )
                }

                Spacer(modifier = Modifier.height(24.dp))

                Text(
                    text = "Pair Your Device",
                    style = MaterialTheme.typography.headlineSmall.copy(
                        fontWeight = FontWeight.Bold,
                        color = Slate100
                    )
                )

                Spacer(modifier = Modifier.height(8.dp))

                Text(
                    text = "Enter the 6-digit pairing code shown on your Nomi dashboard to sync your workspace.",
                    style = MaterialTheme.typography.bodyMedium.copy(
                        color = Slate400,
                        textAlign = TextAlign.Center
                    )
                )

                Spacer(modifier = Modifier.height(32.dp))

                // Pairing Code Input
                OutlinedTextField(
                    value = pairingCode,
                    onValueChange = { input ->
                        val filtered = input.filter { it.isLetterOrDigit() }.uppercase()
                        if (filtered.length <= 6) pairingCode = filtered
                    },
                    modifier = Modifier.fillMaxWidth(),
                    label = { Text("Pairing Code", color = Slate400) },
                    placeholder = { Text("123-ABC", color = Slate700) },
                    leadingIcon = {
                        Icon(
                            imageVector = Lucide.Smartphone,
                            contentDescription = null,
                            tint = Slate400
                        )
                    },
                    shape = RoundedCornerShape(12.dp),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedBorderColor = Indigo500,
                        unfocusedBorderColor = Slate800,
                        unfocusedContainerColor = Slate900,
                        cursorColor = Indigo500,
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White
                    ),
                    visualTransformation = PairingCodeTransformation(),
                    keyboardOptions = KeyboardOptions(
                        keyboardType = KeyboardType.Text,
                        capitalization = KeyboardCapitalization.Characters,
                        imeAction = ImeAction.Done
                    ),
                    singleLine = true,
                    textStyle = MaterialTheme.typography.bodyLarge.copy(
                        letterSpacing = 4.sp,
                        fontWeight = FontWeight.Bold
                    )
                )

                Spacer(modifier = Modifier.height(24.dp))

                // Login Button
                Button(
                    onClick = { viewModel.sendPairingRequest(pairingCode) },
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(52.dp),
                    shape = RoundedCornerShape(12.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = Indigo500,
                        contentColor = Color.White,
                        disabledContainerColor = Indigo500.copy(alpha = 0.5f)
                    ),
                    enabled = pairingCode.length == 6 && state !is LoginState.Loading
                ) {
                    if (state is LoginState.Loading) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(24.dp),
                            color = Color.White,
                            strokeWidth = 2.dp
                        )
                    } else {
                        Text(
                            "Pair Device",
                            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Bold)
                        )
                    }
                }

                Spacer(modifier = Modifier.height(48.dp))

                // Footer info
                Text(
                    text = "Secure pairing powered by Arta AI",
                    style = MaterialTheme.typography.labelSmall.copy(
                        color = Slate700,
                        fontWeight = FontWeight.Normal
                    )
                )
            }
        }
    }
}
