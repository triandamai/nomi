package id.nomi.trianapp.di

import androidx.room.Room
import androidx.room.RoomDatabase
import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.local.getRoomDatabase
import id.nomi.trianapp.data.preferences.PreferencesStorage
import id.nomi.trianapp.data.remote.getHttpClient
import io.ktor.client.HttpClient
import io.ktor.client.engine.darwin.Darwin
import io.ktor.client.plugins.defaultRequest
import kotlinx.cinterop.ExperimentalForeignApi
import org.koin.dsl.module
import platform.Foundation.NSDocumentDirectory
import platform.Foundation.NSFileManager
import platform.Foundation.NSUserDomainMask
import platform.darwin.NSObject


fun getDatabaseBuilder(): RoomDatabase.Builder<NomiDb> {
    val dbFilePath = documentDirectory() + "/my_room.db"
    return Room.databaseBuilder<NomiDb>(
        name = dbFilePath,
    )
}

@OptIn(ExperimentalForeignApi::class)
private fun documentDirectory(): String {
    val documentDirectory = NSFileManager.defaultManager.URLForDirectory(
        directory = NSDocumentDirectory,
        inDomain = NSUserDomainMask,
        appropriateForURL = null,
        create = false,
        error = null,
    )
    return requireNotNull(documentDirectory?.path)
}
actual val platformModule= module {
    single<PreferencesStorage>(definition = { PreferencesStorage(NSObject()) })
    single { getHttpClient(get()) }
    single<NomiDb>(definition = { getRoomDatabase(getDatabaseBuilder()) })
}