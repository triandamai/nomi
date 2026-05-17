package id.nomi.trianapp

import androidx.multidex.MultiDexApplication

import id.nomi.trianapp.di.initKoin
import org.koin.android.ext.koin.androidContext
import org.koin.core.context.startKoin

class NomiApp : MultiDexApplication() {
    override fun onCreate() {
        super.onCreate()
        initKoin {
            androidContext(this@NomiApp)
        }
    }
}