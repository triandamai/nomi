package id.nomi.trianapp.di

import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.data.local.getRoomDatabase
import id.nomi.trianapp.data.preferences.PreferencesStorage
import org.koin.core.module.Module
import org.koin.dsl.module


import androidx.room.Room
import androidx.room.RoomDatabase
import id.nomi.trianapp.data.remote.getHttpClient
import java.io.File

fun getDatabaseBuilder(): RoomDatabase.Builder<NomiDb> {
    val dbFile = File(System.getProperty("java.io.tmpdir"), "arta_local.db")
    return Room.databaseBuilder<NomiDb>(
        name = dbFile.absolutePath,
    )
}

actual val platformModule = module {
    single<PreferencesStorage> {
        PreferencesStorage("DESKTOP")
    }
    single { getHttpClient(get()) }
    single<NomiDb> {
        getRoomDatabase(getDatabaseBuilder())
    }
}