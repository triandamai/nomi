package id.nomi.trianapp

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform