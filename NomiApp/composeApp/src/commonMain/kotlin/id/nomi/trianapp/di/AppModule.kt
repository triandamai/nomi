package id.nomi.trianapp.di

import id.nomi.trianapp.MainViewModel
import id.nomi.trianapp.data.local.NomiDao
import id.nomi.trianapp.data.local.NomiDb
import id.nomi.trianapp.domain.usecase.SendPairingRequestUseCase
import id.nomi.trianapp.ui.screen.auth.LoginViewModel
import org.koin.core.context.startKoin
import org.koin.core.module.Module
import org.koin.core.module.dsl.viewModel
import org.koin.dsl.KoinAppDeclaration
import org.koin.dsl.module

expect val platformModule: Module
val dataModule = module {
    single { SendPairingRequestUseCase(get(), get(),get()) }
}

val viewModelModule = module {
    viewModel {
        MainViewModel(get(), get())
    }
    viewModel { LoginViewModel(get()) }
}

val allModules = listOf(platformModule, networkModule, dataModule, viewModelModule)

fun initKoin(config: KoinAppDeclaration? = null) {
    startKoin {
        config?.invoke(this)
        modules(allModules)
    }
}
