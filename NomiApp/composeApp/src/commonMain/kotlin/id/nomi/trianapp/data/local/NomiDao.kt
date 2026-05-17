package id.nomi.trianapp.data.local

import androidx.room.*
import kotlinx.coroutines.flow.Flow

@Entity(tableName = "profiles")
data class ProfileEntity(
    @PrimaryKey val id: String,
    val displayName: String,
    val avatarUrl: String?,
    val role: String
)

@Entity(tableName = "channels")
data class ChannelEntity(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    val paired: Boolean,
    val platform: String
)

@Entity(tableName = "conversations")
data class ConversationEntity(
    @PrimaryKey val id: String,
    val name: String,
    val cumulativeTokens: Long,
    val createdAt: String,
    val updatedAt: String
)

@Dao
interface NomiDao {
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertProfile(profile: ProfileEntity)

    @Query("SELECT * FROM profiles LIMIT 1")
    fun getProfile(): Flow<ProfileEntity?>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertChannels(channels: List<ChannelEntity>)

    @Query("DELETE FROM channels")
    suspend fun deleteChannels()

    @Query("SELECT * FROM channels")
    fun getChannels(): Flow<List<ChannelEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertConversations(conversations: List<ConversationEntity>)

    @Query("DELETE FROM conversations")
    suspend fun deleteConversations()

    @Query("SELECT * FROM conversations")
    fun getConversations(): Flow<List<ConversationEntity>>
    
    @Transaction
    suspend fun clearAndInsertData(
        profile: ProfileEntity,
        channels: List<ChannelEntity>,
        conversations: List<ConversationEntity>
    ) {
        insertProfile(profile)
        deleteChannels()
        insertChannels(channels)
        deleteConversations()
        insertConversations(conversations)
    }
}
