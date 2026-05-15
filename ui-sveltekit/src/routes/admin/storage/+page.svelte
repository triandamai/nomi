<script lang="ts">
    import {page} from '$app/stores';
    import {goto} from '$app/navigation';
    import {chatApi} from '$lib/api/client';
    import {profileStore} from '$lib/stores/profile.svelte';
    import {popupStore} from '$lib/stores/popup.svelte';
    import {
        Folder,
        File,
        Database,
        ChevronRight,
        AlertCircle,
        RefreshCw,
        Trash2,
        Upload,
        ExternalLink,
        X,
        FileIcon,
        ImageIcon,
        Download
    } from 'lucide-svelte';
    import {env} from '$env/dynamic/public';

    const BASE_URL = env.PUBLIC_GATEWAY_URL || 'http://localhost:8000/api';
    const FILE_URL = BASE_URL.replace('/api', '') + '/api/files';

    let loading = $state(true);
    let uploading = $state(false);
    let items = $state<any[]>([]);
    let error = $state<string | null>(null);
    let selectedFile = $state<any>(null);

    // Drill down logic based on ?path= search param
    let currentPath = $derived($page.url.searchParams.get('path') || '');

    // Role guard
    $effect(() => {
        if (!profileStore.loading && profileStore.currentUser) {
            if (profileStore.currentUser.role !== 'admin') {
                goto('/chat');
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
                items = res.data
            } else {
                error = res.meta?.message || 'Failed to load storage';
            }
        } catch (e: any) {
            error = e.message || 'An error occurred';
        } finally {
            loading = false;
        }
    }

    function stripUrl(full_path: string) {
        if (full_path.startsWith("conversations/")) {
            return full_path.replace("conversations/", "/")
        }
        return full_path
    }

    $effect(() => {
        loadData(currentPath);
    });

    function handleNavigate(path: string) {
        goto(`?path=${encodeURIComponent(path)}`);
    }

    function handleFileClick(item: any) {
        selectedFile = item;
        popupStore.open({
            title: 'File Details',
            width: 'max-w-2xl',
            contentSnippet: fileDetailsContent,
            footerSnippet: fileDetailsFooter
        });
    }

    async function handleDeleteFile(item: any) {
        if (!confirm(`Are you sure you want to delete ${item.name}?`)) return;

        try {
            await chatApi.deleteStorage(item.full_path);
            popupStore.closeLast();
            loadData(currentPath);
        } catch (e: any) {
            alert('Failed to delete file: ' + e.message);
        }
    }

    async function handleUpload(e: Event) {
        const input = e.target as HTMLInputElement;
        if (!input.files?.length) return;
        if (!currentPath) {
            alert('Please enter a bucket first');
            return;
        }

        uploading = true;
        try {
            for (const file of input.files) {
                await chatApi.uploadToStorage(file, currentPath);
            }
            loadData(currentPath);
        } catch (e: any) {
            alert('Upload failed: ' + e.message);
        } finally {
            uploading = false;
            input.value = '';
        }
    }

    function isImage(name: string) {
        return /\.(jpg|jpeg|png|gif|webp|svg|jfif)$/i.test(name);
    }
</script>

{#snippet fileDetailsContent()}
    <div class="space-y-6">
        {#if isImage(selectedFile.name)}
            <div class="rounded-xl overflow-hidden bg-slate-900 border border-slate-800 flex items-center justify-center min-h-[300px] relative group">
                <img
                        src={FILE_URL + stripUrl(selectedFile.full_path)}
                        alt={selectedFile.name}
                        class="max-w-full max-h-[500px] object-contain"
                />
            </div>
        {:else}
            <div class="rounded-xl bg-slate-900 border border-slate-800 p-12 flex flex-col items-center justify-center space-y-4">
                <div class="p-6 bg-slate-950 rounded-2xl border border-slate-800">
                    <FileIcon class="w-16 h-16 text-slate-500"/>
                </div>
                <p class="text-sm text-slate-400">Preview not available for this file type</p>
            </div>
        {/if}

        <div class="grid grid-cols-2 gap-4">
            <div class="space-y-1">
                <p class="text-[10px] uppercase font-bold text-slate-500 tracking-widest">File Name</p>
                <p class="text-sm text-slate-200 font-medium truncate">{selectedFile.name}</p>
            </div>
            <div class="space-y-1">
                <p class="text-[10px] uppercase font-bold text-slate-500 tracking-widest">Full Path</p>
                <p class="text-sm text-slate-400 font-mono truncate">{stripUrl(selectedFile.full_path)}</p>
            </div>
        </div>
    </div>
{/snippet}

{#snippet fileDetailsFooter()}
    <div class="flex justify-between w-full">
        <button
                onclick={() => handleDeleteFile(selectedFile)}
                class="flex items-center gap-2 px-4 py-2 text-xs font-bold uppercase tracking-wider text-rose-500 hover:bg-rose-500/10 rounded-xl transition-all"
        >
            <Trash2 size={14}/>
            Delete File
        </button>
        <div class="flex gap-3">
            <a
                    href={FILE_URL + stripUrl(selectedFile.full_path)}
                    target="_blank"
                    download
                    class="flex items-center gap-2 px-4 py-2 text-xs font-bold uppercase tracking-wider bg-slate-800 hover:bg-slate-700 text-white rounded-xl transition-all"
            >
                <Download size={14}/>
                Download
            </a>
            <button
                    onclick={() => popupStore.closeLast()}
                    class="px-6 py-2 text-xs font-bold uppercase tracking-wider text-slate-400 hover:text-white transition-all"
            >
                Close
            </button>
        </div>
    </div>
{/snippet}

<div class="h-full flex flex-col bg-slate-950 text-slate-100 overflow-hidden">
    <div class="flex items-center p-4 border-b border-slate-800 bg-[#0f172a]/50 backdrop-blur-md shrink-0 gap-4">
        <div class="flex items-center">
            <Database class="w-5 h-5 text-slate-400 mr-3"/>
            <div class="flex items-center space-x-1 text-sm font-medium">
                <button
                        class="hover:text-white transition-colors"
                        class:text-slate-500={currentPath !== ''}
                        onclick={() => goto('?')}
                >
                    Root
                </button>
                {#each breadcrumbs as crumb, i}
                    <ChevronRight class="w-4 h-4 text-slate-600"/>
                    <button
                            class="hover:text-white transition-colors"
                            class:text-slate-500={i !== breadcrumbs.length - 1}
                            onclick={() => handleNavigate(crumb.path)}
                    >
                        {crumb.name}
                    </button>
                {/each}
            </div>
        </div>

        <div class="ml-auto flex items-center gap-2">
            {#if currentPath && currentPath.includes('/')}
                <label class="flex items-center gap-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-xl text-xs font-bold cursor-pointer transition-all shadow-lg shadow-blue-900/20">
                    {#if uploading}
                        <RefreshCw class="w-3.5 h-3.5 animate-spin"/>
                        <span>Uploading...</span>
                    {:else}
                        <Upload class="w-3.5 h-3.5"/>
                        <span>Upload File</span>
                    {/if}
                    <input type="file" multiple class="hidden" onchange={handleUpload} disabled={uploading}/>
                </label>
            {/if}

            <button
                    onclick={() => loadData(currentPath)}
                    class="p-2 rounded-xl hover:bg-slate-800 transition-colors"
                    disabled={loading}
            >
                <RefreshCw class="w-4 h-4 {loading ? 'animate-spin text-slate-400' : 'text-slate-400'}"/>
            </button>
        </div>
    </div>

    <div class="flex-1 overflow-auto p-4">
        {#if error}
            <div class="bg-red-500/10 border border-red-500/20 rounded-xl p-4 flex items-start gap-3">
                <AlertCircle class="w-5 h-5 text-red-500 shrink-0 mt-0.5"/>
                <div>
                    <h3 class="text-sm font-medium text-red-500">Storage Error</h3>
                    <p class="text-xs text-red-400/80 mt-1">{error}</p>
                </div>
            </div>
        {:else if items.length === 0 && !loading}
            <div class="h-full flex flex-col items-center justify-center text-slate-500 space-y-3">
                <Folder class="w-12 h-12 opacity-20"/>
                <p class="text-sm">This folder is empty</p>
                {#if currentPath && currentPath.includes('/')}
                    <p class="text-xs text-slate-600">You can upload files to this folder using the button above.</p>
                {/if}
            </div>
        {:else}
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
                {#each items as item}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div
                            class="group flex flex-col p-4 rounded-2xl border border-slate-800/50 bg-slate-900/50 hover:bg-slate-800 hover:border-slate-700 cursor-pointer transition-all animate-in fade-in zoom-in-95 duration-200"
                            onclick={() => {
                            if (item.type === 'file') {
                                handleFileClick(item);
                            } else {
                                handleNavigate(item.full_path.endsWith('/') ? item.full_path : item.full_path + '/');
                            }
                        }}
                    >
                        <div class="mb-4 aspect-square rounded-xl bg-slate-950 flex items-center justify-center relative overflow-hidden">
                            {#if item.type === 'bucket'}
                                <Database
                                        class="w-10 h-10 text-indigo-500 group-hover:scale-110 transition-transform duration-300"/>
                            {:else if item.type === 'folder'}
                                <Folder class="w-10 h-10 text-amber-500 group-hover:scale-110 transition-transform duration-300"/>
                            {:else if isImage(item.name)}
                                <img
                                        src={FILE_URL + stripUrl(item.full_path)}
                                        alt={item.name}
                                        class="w-full h-full object-cover opacity-60 group-hover:opacity-100 transition-opacity duration-300"
                                        loading="lazy"
                                />
                                <div class="absolute inset-0 bg-gradient-to-t from-slate-950/80 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"/>
                            {:else}
                                <File class="w-10 h-10 text-sky-500 group-hover:scale-110 transition-transform duration-300"/>
                            {/if}
                        </div>

                        <div class="overflow-hidden">
                            <h3 class="text-xs font-bold text-slate-200 truncate group-hover:text-white transition-colors"
                                title={item.name}>
                                {item.name}
                            </h3>
                            <div class="flex items-center gap-2 mt-1">
                                <span class="text-[9px] text-slate-500 uppercase font-black tracking-tighter">
                                    {item.type}
                                </span>
                                {#if item.type === 'file'}
                                    <span class="w-1 h-1 rounded-full bg-slate-800"></span>
                                    <span class="text-[9px] text-slate-600 truncate">
                                        .{item.name.split('.').pop()}
                                    </span>
                                {/if}
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>
