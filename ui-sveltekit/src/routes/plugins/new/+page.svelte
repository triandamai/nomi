<script lang="ts">
    import { onMount } from 'svelte';
    import { chatApi } from '$lib/api/client';
    import { Save, ArrowLeft, Terminal, FileJson, BookOpen, Fingerprint, Play, Loader2, Hash, Edit2 } from 'lucide-svelte';
    import { goto } from '$app/navigation';
    import toast from 'svelte-french-toast';
    import MonacoEditor from '$lib/components/MonacoEditor.svelte';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { generateNomiTypeDefinition } from '$lib/utils/plugin';
    import JsonSchemaEditorPopUp from '$lib/components/JsonSchemaEditorPopUp.svelte';
    import IntentEditorPopUp from '$lib/components/IntentEditorPopUp.svelte';

    let isSaving = $state(false);
    let isExecuting = $state(false);
    let executionOutput = $state<string | null>(null);
    let executionError = $state<string | null>(null);

    let testArgs = $state<Record<string, any>>({});
    let testArgsSchema = $state<Record<string, any>>({});

    let intents = $state<string[]>([]);

    let isResizing = $state(false);
    let consoleHeight = $state(192); // default height in pixels (12rem)

    function startResizing(e: MouseEvent) {
        isResizing = true;
        e.preventDefault();
    }

    function stopResizing() {
        isResizing = false;
    }

    function handleMouseMove(e: MouseEvent) {
        if (!isResizing) return;
        const newHeight = window.innerHeight - e.clientY;
        if (newHeight > 60 && newHeight < window.innerHeight * 0.8) {
            consoleHeight = newHeight;
        }
    }

    let plugin = $state({
        slug: '',
        name: '',
        description: '',
        schema_json: '{\n  "type": "object",\n  "properties": {},\n  "required": []\n}',
        rules_text: '1. Follow standard rules.',
        script_code: `export default async function run(args: NomiArgs) {\n \t// Access your custom payload\n\tconsole.log(args);\n \treturn { message:"Hi From Nomi" };\n}`
    });

    let dynamicTypeDefinition = $derived(generateNomiTypeDefinition(plugin.schema_json));

    function handleOpenTest() {
        try {
            const schema = JSON.parse(plugin.schema_json);
            testArgsSchema = schema.properties || {};

            Object.entries(testArgsSchema).forEach(([key, prop]: [string, any]) => {
                if (prop.type === 'integer' || prop.type === 'number') {
                    testArgs[key] = testArgs[key] ?? 0;
                } else if (prop.type === 'boolean') {
                    testArgs[key] = testArgs[key] ?? false;
                } else {
                    testArgs[key] = testArgs[key] ?? '';
                }
            });

            if (Object.keys(testArgsSchema).length === 0) {
                handleSimulate();
            } else {
                popupStore.open({
                    title: 'Test Run Configuration',
                    width: 'max-w-md',
                    contentSnippet: testArgsSnippet
                });
            }
        } catch (e) {
            toast.error("Invalid JSON Schema format");
        }
    }

    function handleEditSchema() {
        popupStore.open({
            title: 'Schema Architect',
            width: 'max-w-3xl',
            contentSnippet: schemaSnippet
        });
    }

    function handleOpenIntentEditor() {
        popupStore.open({
            title: 'Routing Trigger Architect',
            width: 'max-w-xl',
            contentSnippet: intentSnippet
        });
    }

    async function handleSimulate() {
        popupStore.closeLast();
        isExecuting = true;
        executionOutput = null;
        executionError = null;

        try {
            const res = await chatApi.executeEdgeFunction(plugin.script_code, testArgs);
            if (res.meta.code >= 200 && res.meta.code <= 299) {
                if (res.data.logs) {
                    executionOutput = `[LOGS]\n${res.data.logs}\n\n[RESULT]\n${res.data.result}`;
                } else {
                    executionOutput = res.data.result;
                }
            } else {
                executionError = res.meta.message;
            }
        } catch (e: any) {
            executionError = e.message || "Failed to execute script";
        } finally {
            isExecuting = false;
        }
    }

    async function handleSave() {
        if (!plugin.name || !plugin.slug || !plugin.script_code) {
            toast.error("Name, slug, and code are required.");
            return;
        }

        try {
            let parsedSchema;
            try {
                parsedSchema = JSON.parse(plugin.schema_json);
            } catch (e) {
                toast.error("Invalid JSON Schema format");
                return;
            }

            isSaving = true;
            await chatApi.createEdgeFunction({
                ...plugin,
                schema_json: parsedSchema,
                intents
            });
            toast.success("Plugin Created Successfully!");
            goto('/plugins');
        } catch (e: any) {
            toast.error("Failed to create plugin");
        } finally {
            isSaving = false;
        }
    }

    onMount(() => {
        window.addEventListener('mousemove', handleMouseMove);
        window.addEventListener('mouseup', stopResizing);
        return () => {
            window.removeEventListener('mousemove', handleMouseMove);
            window.removeEventListener('mouseup', stopResizing);
        };
    });
</script>

<div class="flex flex-col h-screen bg-slate-950 text-slate-200">
    <div class="flex h-14 shrink-0 items-center justify-between px-6 bg-slate-900 border-b border-slate-800">
        <div class="flex items-center gap-4">
            <button onclick={() => goto('/plugins')} class="p-2 text-slate-400 hover:text-white transition-colors">
                <ArrowLeft class="w-4 h-4" />
            </button>
            <h1 class="font-black text-sm uppercase tracking-widest">New Edge Plugin</h1>
        </div>
        <button 
            onclick={handleSave} 
            disabled={isSaving}
            class="flex items-center gap-2 bg-emerald-500 hover:bg-emerald-400 text-slate-950 px-4 py-2 rounded-lg font-bold text-xs uppercase tracking-widest transition-colors disabled:opacity-50"
        >
            <Save class="w-4 h-4" />
            {isSaving ? 'Saving...' : 'Deploy Plugin'}
        </button>
    </div>

    <div class="flex-1 flex flex-col lg:flex-row overflow-hidden">
        <!-- Configuration Pane -->
        <div class="w-full lg:w-1/3 border-b lg:border-b-0 lg:border-r border-slate-800 bg-slate-900/50 p-6 overflow-y-auto custom-scrollbar space-y-6">
            <div class="space-y-1.5">
                <label class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-500"><Fingerprint class="w-3 h-3" /> Name</label>
                <input type="text" bind:value={plugin.name} class="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-sm focus:border-sky-500 outline-none transition-colors" placeholder="Crypto Tracker" />
            </div>

            <div class="space-y-1.5">
                <label class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-500"><Terminal class="w-3 h-3" /> Slug (URL-friendly)</label>
                <input type="text" bind:value={plugin.slug} class="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-sm focus:border-sky-500 outline-none transition-colors font-mono text-sky-400" placeholder="crypto_tracker" />
            </div>

            <div class="space-y-1.5">
                <label class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-500"><BookOpen class="w-3 h-3" /> Description (For LLM)</label>
                <textarea bind:value={plugin.description} rows="3" class="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-sm focus:border-sky-500 outline-none transition-colors" placeholder="Fetches current cryptocurrency prices..."></textarea>
            </div>

            <div class="space-y-1.5">
                <div class="flex items-center justify-between">
                    <label class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-500"><Hash class="w-3 h-3" /> Routing Intents</label>
                    <button 
                        onclick={handleOpenIntentEditor}
                        class="flex items-center gap-1.5 text-[9px] font-black uppercase tracking-wider text-sky-400 hover:text-sky-300 transition-colors"
                    >
                        <Edit2 class="w-2.5 h-2.5" />
                        Edit Triggers
                    </button>
                </div>
                <div class="flex flex-wrap gap-1.5 p-3 bg-slate-950 border border-slate-800 rounded-lg min-h-[3rem]">
                    {#if intents.length === 0}
                        <span class="text-[10px] text-slate-700 italic">No triggers configured...</span>
                    {:else}
                        {#each intents as intent}
                            <span class="px-2 py-0.5 rounded-full text-[10px] bg-slate-900 text-emerald-500 border border-emerald-900/30 font-mono">#{intent}</span>
                        {/each}
                    {/if}
                </div>
                <p class="text-[9px] text-slate-600 font-medium ml-1 italic">Semantic triggers for Nomi's routing engine.</p>
            </div>

            <div class="space-y-1.5">
                <div class="flex items-center justify-between">
                    <label class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-500"><FileJson class="w-3 h-3" /> JSON Schema</label>
                    <button 
                        onclick={handleEditSchema}
                        class="flex items-center gap-1.5 text-[9px] font-black uppercase tracking-wider text-sky-400 hover:text-sky-300 transition-colors"
                    >
                        <Edit2 class="w-2.5 h-2.5" />
                        Edit Schema
                    </button>
                </div>
                <div class="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-[11px] font-mono text-sky-300/70 overflow-hidden max-h-48 overflow-y-auto custom-scrollbar whitespace-pre">
                    {plugin.schema_json}
                </div>
            </div>

             <div class="space-y-1.5">
                <label class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-slate-500">Rules & Constraints</label>
                <textarea bind:value={plugin.rules_text} rows="3" class="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-sm focus:border-sky-500 outline-none transition-colors"></textarea>
            </div>
        </div>

        <!-- Code Canvas -->
        <div class="flex-1 flex flex-col bg-[#0d1117] min-h-[50vh] lg:min-h-0 overflow-hidden">
            <div class="h-10 shrink-0 border-b border-slate-800 flex items-center justify-between px-4 bg-slate-900/50">
                <span class="text-[10px] font-mono text-slate-500 uppercase tracking-widest">index.ts (Bun Runtime)</span>
                <button 
                    onclick={handleOpenTest} 
                    disabled={isExecuting}
                    class="flex items-center gap-1.5 bg-slate-800 hover:bg-slate-700 text-sky-400 px-3 py-1 rounded text-[10px] font-bold uppercase tracking-widest transition-colors disabled:opacity-50"
                >
                    {#if isExecuting}
                        <Loader2 class="w-3 h-3 animate-spin" />
                        Running...
                    {:else}
                        <Play class="w-3 h-3 fill-current" />
                        Test Run
                    {/if}
                </button>
            </div>
            
            <div class="flex-1 relative min-h-[200px] overflow-hidden">
                <MonacoEditor bind:value={plugin.script_code} language="typescript" typeDefinition={dynamicTypeDefinition} />
            </div>

            <!-- Console Output -->
            <div class="border-t border-slate-800 bg-slate-950 flex flex-col shrink-0 overflow-hidden" 
                 style="height: {consoleHeight}px;">
                <div class="h-8 shrink-0 border-b border-slate-800/50 flex items-center px-4 bg-slate-900/30 cursor-ns-resize select-none"
                     onmousedown={startResizing} role="slider" aria-label="Console height resizer" aria-valuenow={consoleHeight} tabindex="0">
                    <span class="text-[9px] font-mono text-slate-500 uppercase tracking-widest">Console Output</span>
                </div>
                <div class="flex-1 p-4 font-mono text-xs overflow-y-auto custom-scrollbar whitespace-pre-wrap {executionError ? 'text-rose-400' : 'text-slate-300'}">
                    {#if executionOutput || executionError}
                        {executionOutput || executionError}
                    {:else}
                        <span class="text-slate-700 italic">Ready for execution...</span>
                    {/if}
                </div>
            </div>
        </div>
    </div>
</div>

{#snippet schemaSnippet()}
    <JsonSchemaEditorPopUp 
        bind:schemaJson={plugin.schema_json} 
        onSave={() => popupStore.closeLast()} 
    />
{/snippet}

{#snippet intentSnippet()}
    <IntentEditorPopUp 
        bind:intents={intents} 
        onSave={() => popupStore.closeLast()} 
    />
{/snippet}

{#snippet testArgsSnippet()}
    <div class="space-y-4">
        <p class="text-xs text-slate-400 mb-4">Provide arguments for the execution run:</p>
        {#each Object.entries(testArgsSchema) as [key, prop]}
            <div class="space-y-1.5">
                <label class="text-[10px] font-black uppercase tracking-widest text-slate-500">{key.replace(/_/g, ' ')}</label>
                {#if prop.type === 'string'}
                    <input type="text" bind:value={testArgs[key]} placeholder={prop.description || ''} class="w-full bg-slate-900 border border-slate-800 rounded-md py-2 px-3 text-sm focus:outline-none focus:border-sky-500 transition-all placeholder:text-slate-700" />
                {:else if prop.type === 'integer' || prop.type === 'number'}
                    <input type="number" bind:value={testArgs[key]} class="w-full bg-slate-900 border border-slate-800 rounded-md py-2 px-3 text-sm focus:outline-none focus:border-sky-500 transition-all" />
                {:else if prop.type === 'boolean'}
                    <div class="flex items-center gap-3 bg-slate-900 border border-slate-800 rounded-md p-2">
                        <input type="checkbox" bind:checked={testArgs[key]} class="w-4 h-4 rounded border-slate-700 bg-slate-800 text-sky-500 focus:ring-sky-500" />
                        <span class="text-xs text-slate-500 font-medium">Enable {key.replace(/_/g, ' ')}</span>
                    </div>
                {/if}
            </div>
        {/each}
        
        <button
            onclick={handleSimulate}
            disabled={isExecuting}
            class="w-full mt-6 py-3 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-xs font-black uppercase tracking-[0.2em] transition-all flex items-center justify-center gap-3 shadow-lg shadow-sky-900/20"
        >
            {#if isExecuting}
                <Loader2 class="w-4 h-4 animate-spin" />
                Executing...
            {:else}
                <Play class="w-3.5 h-3.5 fill-current" />
                Run Test
            {/if}
        </button>
    </div>
{/snippet}

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
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
