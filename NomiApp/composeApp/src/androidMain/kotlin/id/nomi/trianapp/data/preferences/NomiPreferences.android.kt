package id.nomi.trianapp.data.preferences

import android.app.Application
import android.content.Context
import android.os.Build
import java.util.UUID

actual typealias NomiPreferencesContext = Application

const val SP_NAME = "kmm_app"

actual fun NomiPreferencesContext.putInt(key: String, value: Int) {
    getSpEditor().putInt(key, value).apply()
}

actual fun NomiPreferencesContext.getInt(key: String, default: Int): Int {
    return getSp().getInt(key, default)
}

actual fun NomiPreferencesContext.putString(key: String, value: String) {
    getSpEditor().putString(key, value).apply()
}

actual fun NomiPreferencesContext.getString(key: String): String? {
    return getSp().getString(key, null)
}

actual fun NomiPreferencesContext.putBool(key: String, value: Boolean) {
    getSpEditor().putBoolean(key, value).apply()
}

actual fun NomiPreferencesContext.getBool(key: String, default: Boolean): Boolean {
    return getSp().getBoolean(key, default)
}

actual fun NomiPreferencesContext.remove(key: String) {
    getSpEditor().remove(key).apply()
}

actual fun NomiPreferencesContext.clear() {
    getSpEditor().clear().commit()
}

private fun NomiPreferencesContext.getSp() = getSharedPreferences(SP_NAME, Context.MODE_PRIVATE)

private fun NomiPreferencesContext.getSpEditor() = getSp().edit()

actual fun NomiPreferencesContext.getDeviceId(): String {
    var deviceId = getSp().getString("device_id", null)
    if (deviceId == null) {
        deviceId = UUID.randomUUID().toString()
        getSpEditor().putString("device_id", deviceId).apply()
    }
    return deviceId
}

actual fun NomiPreferencesContext.getDeviceName(): String {
    return Build.MODEL
}

actual fun NomiPreferencesContext.getDeviceType(): String {
    return "Android"
}