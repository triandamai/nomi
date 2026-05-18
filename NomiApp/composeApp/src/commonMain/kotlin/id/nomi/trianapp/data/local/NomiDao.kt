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

@Entity(tableName = "messages")
data class MessageEntity(
    @PrimaryKey val id: String,
    val conversationId: String,
    val displayName: String,
    val role: String,
    val content: String,
    val totalTokens: Long,
    val answerTokens: Long,
    val promptTokens: Long,
    val thought: String?,
    val imageUrl: String?,
    val videoUrl: String?,
    val audioUrl: String?,
    val documentUrl: String?,
    val stickerUrl: String?,
    val userId: String?,
    val createdAt: String
)

@Dao
interface NomiDao {
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertProfile(profile: ProfileEntity)

    @Query("SELECT * FROM profiles LIMIT 1")
    fun getProfile(): Flow<ProfileEntity?>

    @Query("SELECT * FROM profiles LIMIT 1")
    suspend fun getProfileSync(): ProfileEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertChannels(channels: List<ChannelEntity>)

    @Query("DELETE FROM channels")
    suspend fun deleteChannels()

    @Query("DELETE FROM profiles")
    suspend fun deleteProfile()

    @Query("SELECT * FROM channels")
    fun getChannels(): Flow<List<ChannelEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertConversations(conversations: List<ConversationEntity>)

    @Query("DELETE FROM conversations")
    suspend fun deleteConversations()

    @Query("SELECT * FROM conversations")
    fun getConversations(): Flow<List<ConversationEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertMessages(messages: List<MessageEntity>)

    @Query("SELECT * FROM messages WHERE conversationId = :conversationId ORDER BY createdAt ASC")
    fun getMessages(conversationId: String): Flow<List<MessageEntity>>

    @Query("DELETE FROM messages WHERE conversationId = :conversationId")
    suspend fun deleteMessagesByConversation(conversationId: String)
    
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
