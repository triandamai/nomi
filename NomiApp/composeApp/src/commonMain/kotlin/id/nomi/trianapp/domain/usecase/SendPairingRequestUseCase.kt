package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.data.local.*
import id.nomi.trianapp.data.model.ApiResponse
import id.nomi.trianapp.data.model.PairingRequest
import id.nomi.trianapp.data.model.PairingDto
import id.nomi.trianapp.data.preferences.PreferencesConstant
import id.nomi.trianapp.data.preferences.PreferencesStorage
import id.nomi.trianapp.data.remote.ApiClient

class SendPairingRequestUseCase(
    private val apiClient: ApiClient,
    private val preferencesStorage: PreferencesStorage,
    private val nomiDb: NomiDb
) {
    suspend operator fun invoke(pairingCode: String): ApiResponse<PairingDto> {
        val sanitizedCode = if (pairingCode.length == 6) {
            "${pairingCode.substring(0, 3)}-${pairingCode.substring(3)}"
        } else {
            pairingCode
        }
        val response = apiClient.post<PairingRequest, PairingDto>("/api/auth/pair", PairingRequest(sanitizedCode))

        
        response.data?.let { data ->
            // Save access token
            preferencesStorage.put(PreferencesConstant.SESSION_TOKEN, data.accessToken)
            preferencesStorage.put(PreferencesConstant.ACTIVE_U_ID, data.userId)

            // Save profile, channels, and conversations to Room
            val profileEntity = ProfileEntity(
                id = data.profile.id,
                displayName = data.profile.displayName,
                avatarUrl = data.profile.avatarUrl,
                role = data.profile.role
            )
            
            val channelEntities = data.channels.map {
                ChannelEntity(paired = it.paired, platform = it.platform)
            }
            
            val conversationEntities = data.conversations.map {
                ConversationEntity(
                    id = it.id,
                    name = it.name,
                    cumulativeTokens = it.cumulativeTokens,
                    createdAt = it.createdAt,
                    updatedAt = it.updatedAt
                )
            }
            
            nomiDb.nomiDao().clearAndInsertData(profileEntity, channelEntities, conversationEntities)
        }
        
        return response
    }
}
