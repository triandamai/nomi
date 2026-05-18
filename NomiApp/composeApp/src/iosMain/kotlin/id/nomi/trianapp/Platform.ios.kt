package id.nomi.trianapp

import platform.UIKit.UIDevice

class IOSPlatform: Platform {
    override val name: String = UIDevice.currentDevice.systemName() + " " + UIDevice.currentDevice.systemVersion
}

actual fun getPlatform(): Platform = IOSPlatform()

actual fun formatBytes(bytes: Long): String {
    if (bytes < 1024) return "$bytes B"
    val exp = (kotlin.math.log(bytes.toDouble(), 1024.0)).toInt()
    val pre = "KMGTPE"[exp - 1]
    val value = bytes / kotlin.math.pow(1024.0, exp.toDouble())
    return "${((value * 10).toInt() / 10.0)} ${pre}B"
}