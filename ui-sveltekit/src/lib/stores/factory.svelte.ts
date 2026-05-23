import { chatApi } from '$lib/api/client';
import { eventBus } from '$lib/utils';
import { popupStore } from './popup.svelte';

export interface Proposal {
    slug: string;
    name: string;
    description: string;
    schema_json: any;
    how_it_works: string;
    compiled_code: string | null;
    status: string;
    intents: string[];
    error_logs: string | null;
}

export interface FactoryLog {
    time: string;
    log: string;
    step: string;
}

function createFactoryStore() {
    let proposals = $state<Proposal[]>([]);
    let selectedProposal = $state<Proposal | null>(null);
    let liveLogs = $state<FactoryLog[]>([]);
    let currentStep = $state("idle");
    let activeCodeOutput = $state("");
    let isLoadingProposals = $state(false);

    // Initial listener for evolution telemetry
    eventBus.subscribe('sse-evolution', (event: any) => {
        if (!selectedProposal || event.slug !== selectedProposal.slug) return;

        if (event.log) {
            liveLogs = [...liveLogs, {
                time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
                log: event.log,
                step: event.step || currentStep
            }];
        }
        if (event.step) currentStep = event.step;
        if (event.code) activeCodeOutput = event.code;
        
        if (event.step === "success" || event.step === "failed") {
            reloadProposalsList();
        }
    });

    async function reloadProposalsList() {
        isLoadingProposals = true;
        try {
            const res = await chatApi.getProposals();
            if (res.data) {
                proposals = res.data;
            }
        } catch (e) {
            console.error("FactoryStore: Failed to fetch proposals", e);
        } finally {
            isLoadingProposals = false;
        }
    }

    async function fetchProposalLogs(slug: string) {
        try {
            const res = await chatApi.getProposalLogs(slug);
            if (res.data) {
                const logs = res.data.map((l: any) => ({
                    time: new Date(l.created_at).toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
                    log: l.message,
                    step: l.event_step || "system"
                }));
                liveLogs = logs;
            }
        } catch (e) {
            console.error("FactoryStore: Failed to fetch proposal logs", e);
        }
    }

    function selectProposal(item: Proposal, blueprintReviewSnippet?: any) {
        selectedProposal = item;
        
        // Initialize logs with historical data if available
        let historyLogs: FactoryLog[] = [];
        if (item.error_logs) {
            historyLogs = item.error_logs.split('\n')
                .filter((line: string) => line.trim().length > 0)
                .map((line: string) => ({
                    time: "PAST",
                    log: line,
                    step: "history"
                }));
        }

        liveLogs = [
            ...historyLogs,
            {
                time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
                log: `[MONITOR]: Attaching telemetry listener for [${item.slug}]...`,
                step: "monitor"
            }
        ];
        
        // Also try to fetch persistent system logs if it was selected
        fetchProposalLogs(item.slug);

        currentStep = item.status;
        activeCodeOutput = item.compiled_code || "";

        if (item.status === 'pending' && blueprintReviewSnippet) {
            popupStore.open({
                title: 'Blueprint Review',
                width: 'max-w-2xl',
                contentSnippet: blueprintReviewSnippet
            });
        }
    }

    async function launchBuild(slug: string) {
        try {
            const res = await chatApi.approveProposal(slug);
            if (res.data) {
                proposals = proposals.map(p => p.slug === slug ? { ...p, status: res.data.status } : p);
                const item = proposals.find(p => p.slug === slug);
                if (item) selectProposal(item);
            }
        } catch (e) {
            console.error("FactoryStore: Build failed to launch", e);
        }
    }

    async function deployToProduction(slug: string) {
        liveLogs = [...liveLogs, {
            time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
            log: `[DEPLOYMENT]: Sending hot-patch request to gateway production runtime...`,
            step: "deploy"
        }];
        try {
            const res = await chatApi.deployProposal(slug);
            if (res.meta && res.meta.code === 200) {
              liveLogs = [...liveLogs, {
                time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
                log: `[SUCCESS]: Plugin hot-patched into live edge execution memory!`,
                step: "success"
              }];
              reloadProposalsList();
            } else {
              liveLogs = [...liveLogs, {
                time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }),
                log: `[DEPLOY ERROR]: Execution pass aborted.`,
                step: "failed"
              }];
            }
        } catch (e) {
            console.error("FactoryStore: Deployment error", e);
        }
    }

    async function deleteProposal(slug: string) {
        if (!confirm("Are you sure you want to discard this blueprint?")) return;
        try {
            const res = await chatApi.deleteProposal(slug);
            if (res.meta && res.meta.code === 200) {
                if (selectedProposal?.slug === slug) selectedProposal = null;
                reloadProposalsList();
            }
        } catch (e) {
            console.error("FactoryStore: Deletion failed", e);
        }
    }

    return {
        get proposals() { return proposals; },
        get selectedProposal() { return selectedProposal; },
        get liveLogs() { return liveLogs; },
        get currentStep() { return currentStep; },
        get activeCodeOutput() { return activeCodeOutput; },
        set activeCodeOutput(val: string) { activeCodeOutput = val; },
        get isLoadingProposals() { return isLoadingProposals; },
        
        reloadProposalsList,
        selectProposal,
        launchBuild,
        deployToProduction,
        deleteProposal,
        fetchProposalLogs
    };
}

export const factoryStore = createFactoryStore();
