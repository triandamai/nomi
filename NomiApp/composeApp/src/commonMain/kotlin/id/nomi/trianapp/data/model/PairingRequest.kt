package id.nomi.trianapp.data.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName


@Serializable
data class PairingRequest(
    @SerialName("pairing_code") val pairingCode: String
)
