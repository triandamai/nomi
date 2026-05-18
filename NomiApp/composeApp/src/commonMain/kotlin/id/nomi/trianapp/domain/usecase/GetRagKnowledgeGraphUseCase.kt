package id.nomi.trianapp.domain.usecase

import id.nomi.trianapp.ui.screen.rag.GraphData
import id.nomi.trianapp.ui.screen.rag.GraphEdge
import id.nomi.trianapp.ui.screen.rag.GraphNode
import kotlinx.coroutines.delay

class GetRagKnowledgeGraphUseCase {
    suspend operator fun invoke(): GraphData {
        // Simulate network/db delay
        delay(500)
        
        val nodes = listOf(
            GraphNode("1", "User Manual", "document"),
            GraphNode("2", "Safety Instructions", "chunk"),
            GraphNode("3", "Operating Guide", "chunk"),
            GraphNode("4", "Troubleshooting", "chunk"),
            GraphNode("5", "Technical Specs", "document"),
            GraphNode("6", "Power Requirements", "chunk"),
            GraphNode("7", "Dimensions", "chunk")
        )
        
        val edges = listOf(
            GraphEdge("1", "2"),
            GraphEdge("1", "3"),
            GraphEdge("1", "4"),
            GraphEdge("5", "6"),
            GraphEdge("5", "7"),
            GraphEdge("1", "5") // Related documents
        )
        
        return GraphData(nodes, edges)
    }
}
