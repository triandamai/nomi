<script lang="ts">
    import { chatApi } from '$lib/api/client';
    import { Terminal, Play, Loader2, CheckCircle2, AlertCircle } from 'lucide-svelte';
    import { conversationStore } from '$lib/stores/conversation.svelte';

    interface Property {
        type: string;
        description?: string;
    }

    interface Schema {
        name: string;
        description: string;
        parameters: {
            type: string;
            properties: Record<string, Property>;
            required?: string[];
        };
    }

    let { 
        schema = null as Schema | null,
        scriptCode = null as string | null
    } = $props();

    let formData = $state<Record<string, any>>({});
    let isExecuting = $state(false);
    let executionResult = $state<string | null>(null);
    let executionLogs = $state<string | null>(null);
    let executionError = $state<string | null>(null);

    // Initializer function to prevent infinite loops
    function getInitialData(s: Schema | null) {
        const data: Record<string, any> = {};
        if (s?.parameters?.properties) {
            Object.keys(s.parameters.properties).forEach(key => {
                const prop = s.parameters.properties[key];
                if (prop.type === 'integer' || prop.type === 'number') {
                    data[key] = 0;
                } else if (prop.type === 'boolean') {
                    data[key] = false;
                } else {
                    data[key] = '';
                }
            });
        }
        return data;
    }

    // Update form when schema changes (e.g. on mount)
    $effect(() => {
        if (schema) {
            // Reset state safely
            const initial = getInitialData(schema);
            // We use a single assignment to formData to avoid multiple triggers
            formData = initial;
            executionResult = null;
            executionLogs = null;
            executionError = null;
        }
    });

    async function handleExecute() {
        if (!schema) return;
        
        isExecuting = true;
        executionResult = null;
        executionLogs = null;
        executionError = null;
        
        try {
            if (scriptCode) {
                // 🚀 DIRECT PLAYGROUND EXECUTION (Dry Run)
                // This bypasses the need for the plugin to exist in the database
                const res = await chatApi.executeEdgeFunction(scriptCode, formData);
                if (res.meta.code >= 200 && res.meta.code <= 299) {
                    executionResult = JSON.stringify(res.data.result, null, 2);
                    executionLogs = res.data.logs;
                } else {
                    executionError = res.meta.message;
                }
            } else {
                // STANDARD PRODUCTION EXECUTION
                const res = await chatApi.executeSkill(
                    schema.name, 
                    formData,
                    conversationStore.activeConversationId || undefined
                );
                
                if (res.meta.code >= 200 && res.meta.code <= 299) {
                    executionResult = res.data;
                } else {
                    executionError = res.meta.message;
                }
            }
        } catch (e: any) {
            executionError = e.message || "Execution failed";
        } finally {
            isExecuting = false;
        }
    }
</script>

<div class="flex flex-col bg-slate-950/30 h-full p-6 overflow-y-auto custom-scrollbar">
    {#if schema}
        <div class="space-y-6">
            <div class="space-y-1">
                <div class="flex items-center gap-3">
                    <h2 class="text-xl font-black text-sky-400 uppercase tracking-tighter">{schema.name.replace(/_/g, ' ')}</h2>
                    <span class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-sky-500/10 text-sky-500 border border-sky-500/20 uppercase">
                        {scriptCode ? 'Dry Run Test' : 'Lab Test'}
                    </span>
                </div>
                <p class="text-xs text-slate-300 leading-relaxed italic">{schema.description}</p>
            </div>

            <!-- Tool Testing Form -->
            <div class="space-y-4 pt-4 border-t border-slate-800/50">
                <div class="flex items-center gap-2 mb-4">
                    <Terminal class="w-3.5 h-3.5 text-slate-500" />
                    <span class="text-[10px] font-black uppercase tracking-widest text-slate-500">Parameter Configuration</span>
                </div>

                {#if schema.parameters?.properties}
                    <div class="grid gap-4">
                        {#each Object.entries(schema.parameters.properties) as [key, prop]}
                            <div class="space-y-1.5">
                                <label class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-600 ml-1" for={key}>
                                    {key.replace(/_/g, ' ')}
                                    {#if schema.parameters.required?.includes(key)}
                                        <span class="text-rose-500">*</span>
                                    {/if}
                                </label>
                                
                                {#if prop.type === 'string'}
                                    <input
                                        id={key}
                                        type="text"
                                        bind:value={formData[key]}
                                        placeholder={prop.description || ''}
                                        class="w-full bg-slate-900 border border-slate-800 rounded-md py-2 px-3 text-sm focus:outline-none focus:border-sky-500 transition-all placeholder:text-slate-700"
                                    />
                                {:else if prop.type === 'integer' || prop.type === 'number'}
                                    <input
                                        id={key}
                                        type="number"
                                        bind:value={formData[key]}
                                        class="w-full bg-slate-900 border border-slate-800 rounded-md py-2 px-3 text-sm focus:outline-none focus:border-sky-500 transition-all"
                                    />
                                {:else if prop.type === 'boolean'}
                                    <div class="flex items-center gap-3 bg-slate-900 border border-slate-800 rounded-md p-2">
                                        <input
                                            id={key}
                                            type="checkbox"
                                            bind:checked={formData[key]}
                                            class="w-4 h-4 rounded border-slate-700 bg-slate-800 text-sky-500 focus:ring-sky-500"
                                        />
                                        <span class="text-xs text-slate-500 font-medium">Enable {key.replace(/_/g, ' ')}</span>
                                    </div>
                                {/if}
                                
                                {#if prop.description}
                                    <p class="text-[9px] text-slate-600 font-medium ml-1 italic">{prop.description}</p>
                                {/if}
                            </div>
                        {/each}
                    </div>
                {/if}

                <button
                    onclick={handleExecute}
                    disabled={isExecuting}
                    class="w-full mt-6 py-3 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-xs font-black uppercase tracking-[0.2em] transition-all flex items-center justify-center gap-3 shadow-lg shadow-sky-900/20"
                >
                    {#if isExecuting}
                        <Loader2 class="w-4 h-4 animate-spin" />
                        Executing...
                    {:else}
                        <Play class="w-3.5 h-3.5 fill-current" />
                        Execute Skill
                    {/if}
                </button>
            </div>

            <!-- Result Display -->
            {#if executionResult}
                <div class="space-y-2 animate-in fade-in slide-in-from-bottom-2 duration-300">
                    <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-emerald-500 ml-1">
                        <CheckCircle2 class="w-3 h-3" />
                        Result Output
                    </div>
                    <div class="p-4 bg-emerald-500/5 border border-emerald-500/20 rounded-lg text-xs font-mono text-emerald-300/70 whitespace-pre-wrap overflow-x-auto leading-relaxed border-l-2 border-l-emerald-500">
                        {executionResult}
                    </div>
                </div>
            {/if}

            {#if executionLogs}
                <div class="space-y-2 animate-in fade-in slide-in-from-bottom-2 duration-300 mt-4">
                    <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-sky-500 ml-1">
                        <Terminal class="w-3 h-3" />
                        Execution Logs
                    </div>
                    <div class="p-4 bg-sky-500/5 border border-sky-500/20 rounded-lg text-[10px] font-mono text-sky-300/60 whitespace-pre overflow-x-auto leading-relaxed border-l-2 border-l-sky-500">
                        {executionLogs}
                    </div>
                </div>
            {/if}

            {#if executionError}
                <div class="space-y-2 animate-in fade-in slide-in-from-bottom-2 duration-300">
                    <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-rose-500 ml-1">
                        <AlertCircle class="w-3 h-3" />
                        Execution Failed
                    </div>
                    <div class="p-4 bg-rose-500/5 border border-rose-500/20 rounded-lg text-xs font-mono text-rose-400/80 whitespace-pre-wrap leading-relaxed border-l-2 border-l-rose-500">
                        {executionError}
                    </div>
                </div>
            {/if}
        </div>
    {/if}
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
        height: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: #1e293b;
        border-radius: 10px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: #334155;
    }
</style>
