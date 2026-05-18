package id.nomi.trianapp.data.model

import kotlinx.serialization.*
import kotlinx.serialization.json.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*

@Serializable
data class MetadataDto (
    @SerialName("agent_model")
    val agentModel: String,

    @SerialName("rag_embedding")
    val ragEmbedding: String,

    @SerialName("media_classification")
    val mediaClassification: String,

    @SerialName("media_analyze")
    val mediaAnalyze: String
)
