package id.nomi.trianapp.data.preferences

import java.util.Properties
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.net.InetAddress

actual typealias NomiPreferencesContext = String

private const val PROPERTIES_FILE_NAME = "qazir_session.properties"

private fun getPropertiesFile(): File {
    val userHome = System.getProperty("user.home")
    val appDir = File(userHome, ".qazir")
    if (!appDir.exists()) {
        appDir.mkdirs()
    }
    return File(appDir, PROPERTIES_FILE_NAME)
}

private fun loadProperties(): Properties {
    val properties = Properties()
    val file = getPropertiesFile()
    if (file.exists()) {
        FileInputStream(file).use {
            properties.load(it)
        }
    }
    return properties
}

private fun saveProperties(properties: Properties) {
    val file = getPropertiesFile()
    FileOutputStream(file).use {
        properties.store(it, "Qazir Session Storage")
    }
}

actual fun NomiPreferencesContext.putInt(key: String, value: Int) {
    val properties = loadProperties()
    properties.setProperty(key, value.toString())
    saveProperties(properties)
}

actual fun NomiPreferencesContext.getInt(key: String, default: Int): Int {
    val properties = loadProperties()
    return properties.getProperty(key)?.toIntOrNull() ?: default
}

actual fun NomiPreferencesContext.putString(key: String, value: String) {
    val properties = loadProperties()
    properties.setProperty(key, value)
    saveProperties(properties)
}

actual fun NomiPreferencesContext.getString(key: String): String? {
    val properties = loadProperties()
    return properties.getProperty(key)
}

actual fun NomiPreferencesContext.putBool(key: String, value: Boolean) {
    val properties = loadProperties()
    properties.setProperty(key, value.toString())
    saveProperties(properties)
}

actual fun NomiPreferencesContext.getBool(key: String, default: Boolean): Boolean {
    val properties = loadProperties()
    return properties.getProperty(key)?.toBooleanStrictOrNull() ?: default
}


actual fun NomiPreferencesContext.remove(key: String) {
    val properties = loadProperties()
    properties.remove(key)
    saveProperties(properties)
}

actual fun NomiPreferencesContext.clear() {
    val properties = loadProperties()
    properties.clear()
    saveProperties(properties)
}

actual fun NomiPreferencesContext.getDeviceId(): String {
    // For desktop, we can use a combination of hostname and username
    val hostname = InetAddress.getLocalHost().hostName
    val username = System.getProperty("user.name")
    return "$hostname-$username".hashCode().toString()
}

actual fun NomiPreferencesContext.getDeviceName(): String {
    return InetAddress.getLocalHost().hostName
}

actual fun NomiPreferencesContext.getDeviceType(): String {
    return System.getProperty("os.name")
}