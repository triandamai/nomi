package id.nomi.trianapp

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

        setContent {
            App()
        }
    }

    override fun onResume() {
        super.onResume()
        // Optionally trigger a check to ensure MQTT is connected
        // This is handled by the MainViewModel but calling it here 
        // ensures it's checked every time the app comes to foreground.
    }
}

@Preview
@Composable
fun AppAndroidPreview() {
    App()
}