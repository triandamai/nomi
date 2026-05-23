import { chatApi } from "$lib/api/client";

export interface SrpState {
    slug: string;
    enriched_description: string;
    additional_rules: string[];
    learned_phrases: string[];
}

function createSrpStore() {
    // 🎨 Reactive State
    let state = $state<SrpState>({
        slug: "",
        enriched_description: "Original static definition active. No autonomous optimizations detected.",
        additional_rules: [],
        learned_phrases: []
    });
    
    let simulationInput = $state("");
    let simulationOutput = $state("");
    let isSimulating = $state(false);
    let isLoading = $state(false);
    let availablePlugins = $state<string[]>([]);

    // 🚀 Actions
    async function loadAvailablePlugins() {
        try {
            const response = await chatApi.getAvailablePlugins();
            if (response && response.data) {
                availablePlugins = response.data;
            }
        } catch (e) {
            console.error("SrpStore: Failed to load available plugins", e);
        }
    }

    async function loadState(slug: string) {
        isLoading = true;
        try {
            const response = await chatApi.getSrpState(slug);
            if (response && response.data) {
                state = response.data;
            } else {
                reset(slug);
            }
        } catch (e) {
            console.error("SrpStore: Failed to load SRP state", e);
            reset(slug);
        } finally {
            isLoading = false;
        }
    }

    function reset(slug: string) {
        state = {
            slug,
            enriched_description: "Original static definition active. No autonomous optimizations detected.",
            additional_rules: [],
            learned_phrases: []
        };
        simulationOutput = "";
        simulationInput = "";
    }

    async function runSimulation() {
        if (!simulationInput || !state.slug) return;
        
        isSimulating = true;
        simulationOutput = "Processing alignment pass...";
        
        try {
            const response = await chatApi.testSrp(state.slug, simulationInput);
            
            if (response && response.data && response.data.outcome) {
                simulationOutput = response.data.outcome;
            } else if (response && response.meta && response.meta.message) {
                simulationOutput = `Server Message: ${response.meta.message}`;
            } else {
                simulationOutput = "Simulation completed with no clear outcome trace. Verify backend logs.";
            }
        } catch (e: any) {
            console.error("SrpStore: SRP Simulation error", e);
            simulationOutput = `Simulation Failed: ${e.message || "Unknown error occurred during alignment pass."}`;
        } finally {
            isSimulating = false;
        }
    }

    // 🎁 Public Interface (Functional Svelte 5 pattern)
    return {
        get state() { return state; },
        get simulationInput() { return simulationInput; },
        set simulationInput(val: string) { simulationInput = val; },
        get simulationOutput() { return simulationOutput; },
        get isSimulating() { return isSimulating; },
        get isLoading() { return isLoading; },
        get availablePlugins() { return availablePlugins; },
        
        loadAvailablePlugins,
        loadState,
        reset,
        runSimulation
    };
}

export const srpStore = createSrpStore();
