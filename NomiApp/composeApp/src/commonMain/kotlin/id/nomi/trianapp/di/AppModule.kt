package id.nomi.trianapp.di

import id.nomi.trianapp.MainViewModel
import id.nomi.trianapp.domain.usecase.*
import id.nomi.trianapp.ui.screen.auth.LoginViewModel
import id.nomi.trianapp.ui.screen.chat.ChatViewModel
import id.nomi.trianapp.ui.screen.rag.RagViewModel
import id.nomi.trianapp.ui.screen.workspace.WorkspaceViewModel
import org.koin.core.context.startKoin
import org.koin.core.module.Module
import org.koin.core.module.dsl.viewModel
import org.koin.dsl.KoinAppDeclaration
import org.koin.dsl.module

expect val platformModule: Module
val dataModule = module {
    single { SendPairingRequestUseCase(get(), get(), get()) }
    single { GetConversationsUseCase(get(), get()) }
    single { SetActiveConversationUseCase(get()) }
    single { FetchMessagesUseCase(get(), get()) }
    single { GetRagKnowledgeGraphUseCase() }
    single { FetchRagGraphUseCase(get()) }
    single { SendMessageUseCase(get()) }
    single { GetConversationUseCase(get(), get()) }
    factory { GetProfileUseCase(get(), get()) }
    factory { GetProfileSyncUseCase(get()) }
    factory { LogoutUseCase(get(), get()) }
}

val viewModelModule = module {
    viewModel {
        MainViewModel(get(), get(), get(), get(), get())
    }
    viewModel { LoginViewModel(get()) }
    viewModel { WorkspaceViewModel(get(), get(), get(), get()) }
    viewModel {
        ChatViewModel(get(), get(), get(), get(), get(), get())
    }
    viewModel { RagViewModel(get(), get(), get(), get()) }
}

val allModules = listOf(platformModule, networkModule, dataModule, viewModelModule)

fun initKoin(config: KoinAppDeclaration? = null) {
    startKoin {
        config?.invoke(this)
        modules(allModules)
    }
}
