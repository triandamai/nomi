package id.nomi.trianapp.di

import android.content.Context
import androidx.room.Room
import androidx.room.RoomDatabase
import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.local.getRoomDatabase
import id.nomi.trianapp.data.preferences.PreferencesStorage
import id.nomi.trianapp.data.remote.getHttpClient
import org.koin.android.ext.koin.androidApplication
import org.koin.dsl.module


fun getDatabaseBuilder(context: Context): RoomDatabase.Builder<NomiDb> {
    val appContext = context.applicationContext
    val dbFile = appContext.getDatabasePath("arta_local.db")
    return Room.databaseBuilder<NomiDb>(
        context = appContext,
        name = dbFile.absolutePath
    )
}

actual val platformModule = module {
    single<PreferencesStorage> { PreferencesStorage(androidApplication()) }
    single { getHttpClient(get()) }
    single<NomiDb> {
        getRoomDatabase(getDatabaseBuilder(androidApplication()))
    }
}