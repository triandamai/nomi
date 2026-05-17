package id.nomi.trianapp.data.preferences

import platform.Foundation.NSUserDefaults
import platform.darwin.NSObject

actual typealias NomiPreferencesContext = NSObject

actual fun NomiPreferencesContext.putInt(key: String, value: Int) {
    NSUserDefaults.standardUserDefaults.setInteger(value.toLong(), key)
}

actual fun NomiPreferencesContext.getInt(key: String, default: Int): Int {
    return NSUserDefaults.standardUserDefaults.integerForKey(key).toInt()
}

actual fun NomiPreferencesContext.putString(key: String, value: String) {
    NSUserDefaults.standardUserDefaults.setObject(value, key)
}

actual fun NomiPreferencesContext.getString(key: String): String? {
    return NSUserDefaults.standardUserDefaults.stringForKey(key)
}

actual fun NomiPreferencesContext.putBool(key: String, value: Boolean) {
    NSUserDefaults.standardUserDefaults.setBool(value, key)
}

actual fun NomiPreferencesContext.getBool(key: String, default: Boolean): Boolean {
    return NSUserDefaults.standardUserDefaults.boolForKey(key)
}


actual fun NomiPreferencesContext.remove(key: String) {
    NSUserDefaults.standardUserDefaults.removeObjectForKey(key)
}

actual fun NomiPreferencesContext.clear() {
    val userDefaults = NSUserDefaults.standardUserDefaults
    val dictionary = userDefaults.dictionaryRepresentation()
    dictionary.keys.forEach { key ->
        userDefaults.removeObjectForKey(key as String)
    }
    userDefaults.synchronize()
}

actual fun NomiPreferencesContext.getDeviceId(): String {
    return platform.UIKit.UIDevice.currentDevice.identifierForVendor?.UUIDString ?: ""
}

actual fun NomiPreferencesContext.getDeviceName(): String {
    return platform.UIKit.UIDevice.currentDevice.name
}

actual fun NomiPreferencesContext.getDeviceType(): String {
    return platform.UIKit.UIDevice.currentDevice.model
}