package id.nomi.trianapp.data.local


import androidx.room.ConstructedBy
import androidx.room.Database
import androidx.room.RoomDatabase
import androidx.room.RoomDatabaseConstructor
import androidx.room.TypeConverters
import androidx.sqlite.driver.bundled.BundledSQLiteDriver
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.IO

@Database(
    entities = [
        ProfileEntity::class,
        ChannelEntity::class,
        ConversationEntity::class,
        MessageEntity::class
    ],
    version = 11,
    autoMigrations = [],
    exportSchema = true
)
@ConstructedBy(AppDatabaseConstructor::class)
@TypeConverters(Converters::class)
abstract class NomiDb : RoomDatabase() {
    abstract fun nomiDao(): NomiDao
}

// The Room compiler generates the `actual` implementations.
@Suppress("KotlinNoActualForExpect")
expect object AppDatabaseConstructor : RoomDatabaseConstructor<NomiDb> {
    override fun initialize(): NomiDb
}

fun getRoomDatabase(
    builder: RoomDatabase.Builder<NomiDb>
): NomiDb = builder
    .setDriver(BundledSQLiteDriver())
    .setQueryCoroutineContext(Dispatchers.IO)
    .fallbackToDestructiveMigration(true)
    .build()
