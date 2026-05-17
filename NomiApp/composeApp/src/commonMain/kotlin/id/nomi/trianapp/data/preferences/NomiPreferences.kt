package id.nomi.trianapp.data.preferences

expect class NomiPreferencesContext

expect fun NomiPreferencesContext.putInt(key: String, value: Int)

expect fun NomiPreferencesContext.getInt(key: String, default: Int): Int

expect fun NomiPreferencesContext.putString(key: String, value: String)

expect fun NomiPreferencesContext.getString(key: String) : String?

expect fun NomiPreferencesContext.putBool(key: String, value: Boolean)

expect fun NomiPreferencesContext.getBool(key: String, default: Boolean): Boolean
expect fun NomiPreferencesContext.remove(key: String)
expect fun NomiPreferencesContext.clear()

expect fun NomiPreferencesContext.getDeviceId():String

expect fun NomiPreferencesContext.getDeviceName():String
expect fun NomiPreferencesContext.getDeviceType():String



class PreferencesStorage(
    val context: NomiPreferencesContext?
){
    fun put(key: String, value: Int) {
        context?.putInt(key, value)
    }

    fun put(key: String, value: String) {
        context?.putString(key, value)
    }

    fun put(key: String, value: Boolean) {
        context?.putBool(key, value)
    }

    fun getInt(key: String, default: Int): Int
            =  context?.getInt(key, default) ?:0


    fun getString(key: String) : String?
            =  context?.getString(key)


    fun getBool(key: String, default: Boolean): Boolean =
        context?.getBool(key, default) ?: default

    fun remove(key: String) = context?.remove(key)

    fun clear() = context?.clear()

    fun getDeviceId():String = context?.getDeviceId() ?:""
    fun getDeviceName():String = context?.getDeviceName() ?:""
    fun getDeviceType():String = context?.getDeviceType() ?:""

    companion object {
        fun getInstance(context: NomiPreferencesContext?): PreferencesStorage {
            return PreferencesStorage(context)
        }
    }
}
