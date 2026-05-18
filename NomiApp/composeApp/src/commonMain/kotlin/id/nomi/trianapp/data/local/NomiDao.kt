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
    val maxTokenUsage: Long,
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

@Entity(tableName = "reminders")
data class ReminderEntity(
    @PrimaryKey val id: String,
    val taskType: String,
    val content: String,
    val dueAt: String,
    val frequency: String,
    val status: String,
    val userDisplayName: String,
    val conversationTitle: String,
    val createdAt: String
)

@Entity(tableName = "money_tracking")
data class MoneyTrackingEntity(
    @PrimaryKey val id: String,
    val merchantName: String,
    val category: String,
    val description: String?,
    val totalAmount: Long,
    val createdAt: String,
    val itemsJson: String, // JSON representation of List<TransactionItemDto>
    val userDisplayName: String,
    val conversationTitle: String
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

    @Query("SELECT * FROM conversations WHERE id = :id")
    fun getConversation(id: String): Flow<ConversationEntity?>

    @Query("SELECT * FROM conversations WHERE id = :id")
    suspend fun getConversationSync(id: String): ConversationEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertMessages(messages: List<MessageEntity>)

    @Query("SELECT * FROM messages WHERE conversationId = :conversationId ORDER BY createdAt ASC")
    fun getMessages(conversationId: String): Flow<List<MessageEntity>>

    @Query("DELETE FROM messages WHERE conversationId = :conversationId")
    suspend fun deleteMessagesByConversation(conversationId: String)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertReminders(reminders: List<ReminderEntity>)

    @Query("SELECT * FROM reminders ORDER BY dueAt ASC")
    fun getReminders(): Flow<List<ReminderEntity>>

    @Query("DELETE FROM reminders")
    suspend fun deleteReminders()

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertMoneyTracking(items: List<MoneyTrackingEntity>)

    @Query("SELECT * FROM money_tracking ORDER BY createdAt DESC")
    fun getMoneyTracking(): Flow<List<MoneyTrackingEntity>>

    @Query("DELETE FROM money_tracking")
    suspend fun deleteMoneyTracking()
    
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
