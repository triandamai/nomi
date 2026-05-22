import { chatApi } from "$lib/api/client";

export interface SrpState {
    slug: string;
    enriched_description: string;
    additional_rules: string[];
    learned_phrases: string[];
}

class SrpStore {
    state = $state<SrpState>({
        slug: "",
        enriched_description: "Original static definition active. No autonomous optimizations detected.",
        additional_rules: [],
        learned_phrases: []
    });
    
    simulationInput = $state("");
    simulationOutput = $state("");
    isSimulating = $state(false);
    isLoading = $state(false);
    availablePlugins = $state<string[]>([]);

    async loadAvailablePlugins() {
        try {
            const response = await chatApi.getAvailablePlugins();
            if (response && response.data) {
                this.availablePlugins = response.data;
            }
        } catch (e) {
            console.error("Failed to load available plugins:", e);
        }
    }

    async loadState(slug: string) {
        this.isLoading = true;
        try {
            const response = await chatApi.getSrpState(slug);
            if (response && response.data) {
                this.state = response.data;
            } else {
                this.reset(slug);
            }
        } catch (e) {
            console.error("Failed to load SRP state:", e);
            this.reset(slug);
        } finally {
            this.isLoading = false;
        }
    }

    reset(slug: string) {
        this.state = {
            slug,
            enriched_description: "Original static definition active. No autonomous optimizations detected.",
            additional_rules: [],
            learned_phrases: []
        };
        this.simulationOutput = "";
        this.simulationInput = "";
    }

    async runSimulation() {
        if (!this.simulationInput || !this.state.slug) return;
        
        this.isSimulating = true;
        this.simulationOutput = "Processing alignment pass...";
        
        try {
            const response = await chatApi.testSrp(this.state.slug, this.simulationInput);
            
            if (response && response.data && response.data.outcome) {
                this.simulationOutput = response.data.outcome;
            } else if (response && response.meta && response.meta.message) {
                this.simulationOutput = `Server Message: ${response.meta.message}`;
            } else {
                this.simulationOutput = "Simulation completed with no clear outcome trace. Verify backend logs.";
            }
        } catch (e: any) {
            console.error("SRP Simulation error:", e);
            this.simulationOutput = `Simulation Failed: ${e.message || "Unknown error occurred during alignment pass."}`;
        } finally {
            this.isSimulating = false;
        }
    }
}

export const srpStore = new SrpStore();
