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

    let selectedMonth = $state(0); // 0 means All Time
    let selectedYear = $state(0);

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
        get selectedMonth() { return selectedMonth; },
        set selectedMonth(val: number) { selectedMonth = val; },
        get selectedYear() { return selectedYear; },
        set selectedYear(val: number) { selectedYear = val; },

        async fetchGraph(conversationId?: string) {
            loading = true;
            error = null;
            try {
                // Pass 0 or undefined to backend to trigger 'All Time' logic
                const result = await chatApi.getGraph(
                    conversationId, 
                    selectedMonth || undefined, 
                    selectedYear || undefined
                );
                if (result.data) {
                    graphData = result.data;
                } else {
                    error = result.meta?.message || 'Failed to fetch graph';
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
