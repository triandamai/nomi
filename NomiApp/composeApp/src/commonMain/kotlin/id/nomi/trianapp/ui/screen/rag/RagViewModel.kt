package id.nomi.trianapp.ui.screen.rag

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import id.nomi.trianapp.data.local.ConversationEntity
import id.nomi.trianapp.data.model.RagGraphData
import id.nomi.trianapp.data.model.RagNodeDto
import id.nomi.trianapp.data.preferences.PreferencesConstant
import id.nomi.trianapp.data.preferences.PreferencesStorage
import id.nomi.trianapp.domain.usecase.FetchRagGraphUseCase
import id.nomi.trianapp.domain.usecase.GetConversationUseCase
import id.nomi.trianapp.domain.usecase.GetRagKnowledgeGraphUseCase
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

@Serializable
data class GraphData(
    val nodes: List<GraphNode>,
    val edges: List<GraphEdge>
)

@Serializable
data class GraphNode(
    val id: String,
    val label: String,
    val type: String // "document" or "chunk"
)

@Serializable
data class GraphEdge(
    val from: String,
    val to: String
)

@Serializable
data class ThreeDGraphData(
    val nodes: List<ThreeDGraphNode>,
    val links: List<ThreeDGraphLink>
)

@Serializable
data class ThreeDGraphNode(
    val id: String,
    val name: String,
    val `val`: Int,
    val type: String
)

@Serializable
data class ThreeDGraphLink(
    val source: String,
    val target: String
)

class RagViewModel(
    savedStateHandle: SavedStateHandle,
    private val getRagKnowledgeGraphUseCase: GetRagKnowledgeGraphUseCase,
    private val fetchRagGraphUseCase: FetchRagGraphUseCase,
    private val getConversationUseCase: GetConversationUseCase
) : ViewModel() {
    private val _graphData = MutableStateFlow("")
    val graphData: StateFlow<String> = _graphData.asStateFlow()

    private val _isLoadingDetails = MutableStateFlow(false)
    val isLoadingDetails: StateFlow<Boolean> = _isLoadingDetails.asStateFlow()

    private val _selectedNodeDetails = MutableStateFlow<RagNodeDto?>(null)
    val selectedNodeDetails: StateFlow<RagNodeDto?> = _selectedNodeDetails.asStateFlow()

    private val _activeConversation = MutableStateFlow<ConversationEntity?>(null)
    val activeConversation: StateFlow<ConversationEntity?> = _activeConversation.asStateFlow()

    // Keep cache of last fetched data to find nodes
    private var lastGraphData: RagGraphData? = null


    fun setConversationId(conversationId: String?) {
        if (conversationId != null) {
            loadGraphData(conversationId)
            loadConversation(conversationId)
        }
    }

    private fun loadConversation(conversationId: String?) {
        viewModelScope.launch {
            if (conversationId != null) {
                getConversationUseCase(conversationId)
                getConversationUseCase.getLocalConversation(conversationId).collect {
                    _activeConversation.value = it
                }
            }
        }
    }

    private fun loadGraphData(conversationId: String?) {
        viewModelScope.launch {
            if (conversationId != null) {
                fetchRagGraphUseCase(conversationId).onSuccess { data ->
                    lastGraphData = data
                    _graphData.value = Json.encodeToString(RagGraphData.serializer(), data)
                }.onFailure {
                    loadMockGraphData()
                }
            } else {
                loadMockGraphData()
            }
        }
    }

    fun loadNodeDetails(nodeId: String) {
        viewModelScope.launch {
            _isLoadingDetails.value = true
            // Simulate API latency for fetching full node metadata
            delay(600)

            val node = lastGraphData?.nodes?.find { it.id == nodeId }
            if (node != null) {
                _selectedNodeDetails.value = node
            } else {
                // Mockup details if not found in last fetched data (for local testing)
                _selectedNodeDetails.value = RagNodeDto(
                    id = nodeId,
                    label = "Node $nodeId",
                    nodeType = if (nodeId.toIntOrNull()?.rem(2) == 0) "document" else "chunk",
                    color = "#fbbf24"
                )
            }
            _isLoadingDetails.value = false
        }
    }

    fun dismissNodeDetails() {
        _selectedNodeDetails.value = null
    }

    private suspend fun loadMockGraphData() {
        val originalData = getRagKnowledgeGraphUseCase()

        val nodes = originalData.nodes.map {
            ThreeDGraphNode(
                id = it.id,
                name = it.label,
                `val` = if (it.type == "document") 15 else 8,
                type = it.type
            )
        }
        val links = originalData.edges.map {
            ThreeDGraphLink(source = it.from, target = it.to)
        }

        val threeDData = ThreeDGraphData(nodes, links)
        _graphData.value = Json.encodeToString(threeDData)
    }
}
