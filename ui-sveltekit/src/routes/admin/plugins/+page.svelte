<script lang="ts">
    import { onMount } from 'svelte';
    import { chatApi } from '$lib/api/client';
    import { Code2, Plus, Search, Trash2, Cpu, FileJson, Clock } from 'lucide-svelte';
    import { formatDate } from '$lib/utils';
    import Header from '$lib/components/Header.svelte';
    import { goto } from '$app/navigation';
    import toast from 'svelte-french-toast';

    let plugins = $state<any[]>([]);
    let isLoading = $state(true);
    let searchQuery = $state('');

    let filteredPlugins = $derived(
        plugins.filter(p => 
            p.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
            p.slug.toLowerCase().includes(searchQuery.toLowerCase())
        )
    );

    async function loadPlugins() {
        isLoading = true;
        try {
            const res = await chatApi.getEdgeFunctions();
            plugins = res.data;
        } catch (e: any) {
            toast.error("Failed to load plugins");
        } finally {
            isLoading = false;
        }
    }

    async function deletePlugin(slug: string) {
        if (!confirm(`Are you sure you want to delete plugin '${slug}'?`)) return;
        
        try {
            await chatApi.deleteEdgeFunction(slug);
            toast.success("Plugin deleted successfully");
            await loadPlugins();
        } catch (e: any) {
            toast.error("Failed to delete plugin");
        }
    }

    onMount(() => {
        loadPlugins();
    });
</script>

<div class="flex flex-col h-screen bg-slate-950 text-slate-200">
    <Header />

    <main class="flex-1 overflow-y-auto p-6">
        <div class="max-w-7xl mx-auto space-y-6">
            
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-black uppercase tracking-tighter text-sky-400">Dynamic Edge Plugins</h1>
                    <p class="text-xs text-slate-400 font-medium">Serverless TypeScript execution engine.</p>
                </div>

                <button 
                    onclick={() => goto('/admin/plugins/new')}
                    class="flex items-center gap-2 bg-sky-500 hover:bg-sky-400 text-slate-950 px-4 py-2 rounded-lg font-bold text-xs uppercase tracking-widest transition-colors"
                >
                    <Plus class="w-4 h-4" />
                    New Plugin
                </button>
            </div>

            <div class="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden">
                <div class="p-4 border-b border-slate-800 flex items-center justify-between">
                    <div class="relative w-full max-w-sm">
                        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
                        <input
                            type="text"
                            bind:value={searchQuery}
                            placeholder="Search plugins..."
                            class="w-full bg-slate-950 border border-slate-800 rounded-md py-2 pl-9 pr-4 text-sm focus:outline-none focus:border-sky-500 transition-colors"
                        />
                    </div>
                </div>

                {#if isLoading}
                    <div class="p-12 flex justify-center">
                        <Cpu class="w-8 h-8 text-sky-500 animate-pulse" />
                    </div>
                {:else if filteredPlugins.length === 0}
                    <div class="p-12 flex flex-col items-center justify-center text-slate-500 space-y-2">
                        <Code2 class="w-12 h-12 opacity-20" />
                        <p class="text-sm font-medium">No edge plugins found.</p>
                    </div>
                {:else}
                    <div class="overflow-x-auto">
                        <table class="w-full text-left border-collapse">
                            <thead>
                                <tr class="bg-slate-950/50 border-b border-slate-800">
                                    <th class="p-4 text-xs font-black text-slate-500 uppercase tracking-widest">Name / Slug</th>
                                    <th class="p-4 text-xs font-black text-slate-500 uppercase tracking-widest">Description</th>
                                    <th class="p-4 text-xs font-black text-slate-500 uppercase tracking-widest text-center">Version</th>
                                    <th class="p-4 text-xs font-black text-slate-500 uppercase tracking-widest text-right">Actions</th>
                                </tr>
                            </thead>
                            <tbody>
                                {#each filteredPlugins as plugin}
                                    <tr class="border-b border-slate-800/50 hover:bg-slate-800/30 transition-colors group">
                                        <td class="p-4">
                                            <div class="font-bold text-slate-200">{plugin.name}</div>
                                            <div class="text-[10px] font-mono text-sky-400 mt-1">{plugin.slug}</div>
                                        </td>
                                        <td class="p-4">
                                            <div class="text-sm text-slate-400 line-clamp-1 max-w-xs">{plugin.description}</div>
                                            <div class="flex flex-wrap gap-1 mt-2">
                                                {#each plugin.intents || [] as intent}
                                                    <span class="px-1.5 py-0.5 rounded-full text-[9px] bg-slate-950 text-emerald-500 border border-emerald-900/30 font-mono lowercase">#{intent}</span>
                                                {/each}
                                            </div>
                                        </td>
                                        <td class="p-4 text-center">
                                            <span class="inline-flex items-center justify-center px-2 py-1 rounded bg-slate-800 text-xs font-bold text-slate-300">
                                                v{plugin.version}
                                            </span>
                                        </td>
                                        <td class="p-4">
                                            <div class="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                                <button 
                                                    onclick={() => goto(`/admin/plugins/${plugin.slug}`)}
                                                    class="p-2 text-slate-400 hover:text-sky-400 hover:bg-slate-800 rounded transition-colors"
                                                    title="Edit Plugin"
                                                >
                                                    <Code2 class="w-4 h-4" />
                                                </button>
                                                <button 
                                                    onclick={() => deletePlugin(plugin.slug)}
                                                    class="p-2 text-slate-400 hover:text-rose-400 hover:bg-slate-800 rounded transition-colors"
                                                    title="Delete Plugin"
                                                >
                                                    <Trash2 class="w-4 h-4" />
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                {/if}
            </div>
        </div>
    </main>
</div>
