<script lang="ts">
    import { page } from '$app/stores';
    import { goto } from '$app/navigation';
    import { chatApi } from '$lib/api/client';
    import { profileStore } from '$lib/stores/profile.svelte';
    import { Folder, File, Database, ChevronRight, AlertCircle, RefreshCw } from 'lucide-svelte';

    let loading = $state(true);
    let items = $state<any[]>([]);
    let error = $state<string | null>(null);
    
    // Drill down logic based on ?path= search param
    let currentPath = $derived($page.url.searchParams.get('path') || '');

    // Role guard
    $effect(() => {
        if (!profileStore.loading && profileStore.currentUser) {
            if (profileStore.currentUser.role !== 'admin') {
                goto('/dashboard');
            }
        }
    });

    // Breadcrumbs
    let breadcrumbs = $derived.by(() => {
        if (!currentPath) return [];
        const parts = currentPath.split('/').filter(Boolean);
        return parts.map((part, index) => {
            return {
                name: part,
                path: parts.slice(0, index + 1).join('/') + '/'
            };
        });
    });

    async function loadData(path: string) {
        loading = true;
        error = null;
        try {
            const res = await chatApi.exploreStorage(path);
            if (res.data) {
                items = res.data;
            } else {
                error = res.meta?.message || 'Failed to load storage';
            }
        } catch (e: any) {
            error = e.message || 'An error occurred';
        } finally {
            loading = false;
        }
    }

    $effect(() => {
        loadData(currentPath);
    });

    function handleNavigate(path: string) {
        goto(`?path=${encodeURIComponent(path)}`);
    }

    function handleFileClick(fullPath: string) {
        // e.g. /files/{path}
        const baseUrl = chatApi.getProfile.toString().includes('http') ? '' : ''; // In UI, base API url is available inside client.ts but we can just use the public gateway URL. Wait, the frontend API requests go to /api which is proxied by Vite or handled directly.
        // We'll rely on the existing routing proxy or assume standard /api/files/ prefix
        window.open(`/api/files/${fullPath}`, '_blank');
    }
</script>

<div class="h-full flex flex-col bg-zinc-950 text-zinc-100 overflow-hidden">
    <div class="flex items-center p-4 border-b border-zinc-800 bg-zinc-900 shrink-0">
        <Database class="w-5 h-5 text-zinc-400 mr-3" />
        <div class="flex items-center space-x-1 text-sm font-medium">
            <button 
                class="hover:text-white transition-colors"
                class:text-zinc-500={currentPath !== ''}
                onclick={() => goto('?')}
            >
                Root
            </button>
            {#each breadcrumbs as crumb, i}
                <ChevronRight class="w-4 h-4 text-zinc-600" />
                <button 
                    class="hover:text-white transition-colors"
                    class:text-zinc-500={i !== breadcrumbs.length - 1}
                    onclick={() => handleNavigate(crumb.path)}
                >
                    {crumb.name}
                </button>
            {/each}
        </div>
        <button 
            onclick={() => loadData(currentPath)}
            class="ml-auto p-2 rounded-md hover:bg-zinc-800 transition-colors"
            disabled={loading}
        >
            <RefreshCw class="w-4 h-4 {loading ? 'animate-spin text-zinc-400' : 'text-zinc-400'}" />
        </button>
    </div>

    <div class="flex-1 overflow-auto p-4">
        {#if error}
            <div class="bg-red-500/10 border border-red-500/20 rounded-lg p-4 flex items-start gap-3">
                <AlertCircle class="w-5 h-5 text-red-500 shrink-0 mt-0.5" />
                <div>
                    <h3 class="text-sm font-medium text-red-500">Storage Error</h3>
                    <p class="text-xs text-red-400/80 mt-1">{error}</p>
                </div>
            </div>
        {:else if items.length === 0 && !loading}
            <div class="h-full flex flex-col items-center justify-center text-zinc-500 space-y-3">
                <Folder class="w-12 h-12 opacity-20" />
                <p class="text-sm">This folder is empty</p>
            </div>
        {:else}
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
                {#each items as item}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div 
                        class="group flex items-center p-3 rounded-xl border border-zinc-800/50 bg-zinc-900/50 hover:bg-zinc-800 hover:border-zinc-700 cursor-pointer transition-all"
                        onclick={() => {
                            if (item.type === 'file') {
                                handleFileClick(item.full_path);
                            } else {
                                handleNavigate(item.full_path.endsWith('/') ? item.full_path : item.full_path + '/');
                            }
                        }}
                    >
                        {#if item.type === 'bucket'}
                            <Database class="w-8 h-8 text-indigo-400 p-1.5 bg-indigo-500/10 rounded-lg mr-3 group-hover:scale-110 transition-transform" />
                        {:else if item.type === 'folder'}
                            <Folder class="w-8 h-8 text-amber-400 p-1.5 bg-amber-500/10 rounded-lg mr-3 group-hover:scale-110 transition-transform" />
                        {:else}
                            <File class="w-8 h-8 text-sky-400 p-1.5 bg-sky-500/10 rounded-lg mr-3 group-hover:scale-110 transition-transform" />
                        {/if}
                        
                        <div class="overflow-hidden">
                            <h3 class="text-sm font-medium text-zinc-200 truncate" title={item.name}>
                                {item.name}
                            </h3>
                            <p class="text-[10px] text-zinc-500 uppercase tracking-wider mt-0.5">
                                {item.type}
                            </p>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>
