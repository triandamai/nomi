import {chatApi} from '$lib/api/client';

export interface Node {
    id: string;
    label: string;
    node_type: string;
    color?: string;
    conversation_id?: string;
    x?: number;
    y?: number;
}

export interface Link {
    source: string | Node;
    target: string | Node;
    relationship: string;
}

export interface GraphData {
    nodes: Node[];
    links: Link[];
}

function createRagStore() {
    let graphData = $state<GraphData>({nodes: [], links: []});
    let loading = $state(false);
    let error = $state<string | null>(null);
    let searchResults = $state<Node[]>([]);
    let isSearching = $state(false);

    return {
        get graphData() {
            return graphData
        },
        get loading() {
            return loading
        },
        get error() {
            return error
        },
        get searchResults() {
            return searchResults
        },
        get isSearching() {
            return isSearching
        },
        async fetchGraph(conversationId?: string) {
            loading = true;
            error = null;
            console.log("start fetching graph", conversationId)
            try {
                const result = await chatApi.getGraph(conversationId);
                if (result.data) {
                    graphData = result.data;
                } else {
                    error = result.message || 'Failed to fetch graph';
                }
            } catch (e: any) {
                console.error('Failed to fetch graph data:', e);
                error = e.message || 'An error occurred while fetching graph data';
            } finally {
                loading = false;
            }
        },
        async searchGraph(query: string) {
            if (!query.trim()) {
                searchResults = [];
                return;
            }
            isSearching = true;
            try {
                const result = await chatApi.searchGraph(query);
                if (result.data) {
                    searchResults = result.data;
                }
            } catch (e) {
                console.error('Failed to search graph:', e);
            } finally {
                isSearching = false;
            }
        },
        clearSearch() {
            searchResults = [];
        }
    }
}

export const ragStore = createRagStore();
