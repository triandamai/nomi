package id.nomi.trianapp

import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import id.nomi.trianapp.di.allModules
import id.nomi.trianapp.ui.NomiTheme
import id.nomi.trianapp.ui.screen.MainApp
import org.koin.compose.KoinApplication

@Composable
@Preview
fun App() {
    KoinApplication(application = {
        modules(allModules)
    }) {
        NomiTheme {
            MainApp()
        }
    }
}
