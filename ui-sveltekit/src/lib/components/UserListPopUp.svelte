<script lang="ts">
    import { onMount } from 'svelte';
    import { adminStore } from '$lib/stores/admin.svelte';
    import { ChevronDown, Loader2, User, Search, Calendar, ShieldCheck, Mail, Fingerprint } from 'lucide-svelte';

    let searchTimeout: ReturnType<typeof setTimeout>;

    onMount(() => {
        adminStore.fetchUsers();
    });

    function handleSearch(e: Event) {
        const val = (e.target as HTMLInputElement).value;
        adminStore.userSearchQuery = val;
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            adminStore.resetUsers();
            adminStore.fetchUsers();
        }, 300);
    }

    function formatDate(dateStr: string | null) {
        if (!dateStr) return 'N/A';
        return new Date(dateStr).toLocaleString('id-ID', {
            day: '2-digit',
            month: 'short',
            year: 'numeric'
        });
    }
</script>

<div class="flex flex-col h-full text-slate-200">
    <div class="p-4 md:p-6 border-b border-slate-800 bg-[#0f172a]/50 backdrop-blur-md">
        <div class="relative">
            <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
            <input 
                type="text" 
                placeholder="Search by name, email..." 
                oninput={handleSearch}
                value={adminStore.userSearchQuery}
                class="w-full bg-slate-900 border border-slate-800 rounded-2xl pl-11 pr-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-purple-500/50 transition-all placeholder:text-slate-600"
            />
        </div>
    </div>

    <div class="flex-1 overflow-y-auto p-3 md:p-6 space-y-3 custom-scrollbar">
        {#if adminStore.userLoading && adminStore.users.length === 0}
            <div class="flex flex-col items-center justify-center py-24 gap-4">
                <Loader2 class="w-8 h-8 animate-spin text-purple-500" />
                <p class="text-[10px] font-black uppercase tracking-widest text-slate-600">Syncing Directory...</p>
            </div>
        {:else if adminStore.users.length === 0}
            <div class="text-center py-24 bg-slate-900/20 rounded-3xl border border-dashed border-slate-800 mx-2">
                <User class="w-10 h-10 text-slate-800 mx-auto mb-4" />
                <p class="text-sm text-slate-500 px-4">No users found.</p>
            </div>
        {:else}
            <div class="grid gap-3">
                {#each adminStore.users as user (user.id)}
                    <div class="bg-slate-900/40 border border-slate-800/50 rounded-2xl p-4 md:p-5 hover:border-slate-700 transition-all group">
                        <div class="flex flex-col gap-4">
                            <div class="flex flex-col sm:flex-row justify-between items-start gap-3">
                                <div class="flex items-center gap-4 min-w-0">
                                    <div class="w-12 h-12 rounded-2xl bg-slate-800 border border-slate-700 flex items-center justify-center shrink-0 group-hover:border-purple-500/30 transition-colors">
                                        <span class="text-lg font-black text-slate-500 group-hover:text-purple-400">
                                            {(user.display_name || user.name || '?').charAt(0).toUpperCase()}
                                        </span>
                                    </div>
                                    <div class="min-w-0">
                                        <h3 class="font-bold text-slate-100 truncate text-sm md:text-base mb-1">
                                            {user.display_name || user.name || 'Anonymous User'}
                                            {#if user.role === 'admin'}
                                                <span class="ml-2 px-1.5 py-0.5 text-[8px] bg-amber-500/10 text-amber-500 border border-amber-500/20 rounded uppercase font-black tracking-tighter">Admin</span>
                                            {/if}
                                        </h3>
                                        <div class="flex items-center gap-2 text-[10px] text-slate-500">
                                            <Mail size={12} class="shrink-0" />
                                            <span class="truncate">{user.email || 'No email associated'}</span>
                                        </div>
                                    </div>
                                </div>
                                <div class="shrink-0 flex sm:flex-col items-end gap-2">
                                    {#if user.is_verified}
                                        <div class="flex items-center gap-1.5 px-2 py-1 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                                            <ShieldCheck size={10} />
                                            <span class="text-[8px] font-black uppercase tracking-widest">Verified</span>
                                        </div>
                                    {/if}
                                    <div class="flex items-center gap-1.5 text-[9px] text-slate-600 font-mono">
                                        <Calendar size={10} />
                                        <span>Joined {formatDate(user.created_at)}</span>
                                    </div>
                                </div>
                            </div>
                            
                            <div class="pt-3 border-t border-slate-800/50 flex items-center justify-between">
                                <div class="flex items-center gap-2">
                                    <Fingerprint size={12} class="text-slate-600" />
                                    <span class="text-[9px] font-mono text-slate-500 uppercase tracking-widest truncate max-w-[150px]">
                                        {user.id}
                                    </span>
                                </div>
                                <div class="text-[9px] text-slate-600 italic">
                                    Role: <span class="text-slate-400 font-bold uppercase">{user.role || 'User'}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                {/each}
            </div>

            {#if adminStore.hasMoreUsers}
                <div class="pt-4 flex justify-center">
                    <button onclick={() => adminStore.fetchUsers(true)} disabled={adminStore.userLoading} class="flex items-center gap-2 px-6 py-2 bg-slate-900 hover:bg-slate-800 border border-slate-800 rounded-xl text-[10px] font-black uppercase tracking-widest text-slate-400 hover:text-slate-200 transition-all active:scale-95 disabled:opacity-50">
                        {#if adminStore.userLoading} <Loader2 size={14} class="animate-spin" /> Loading... {:else} <ChevronDown size={14} /> Load More Users {/if}
                    </button>
                </div>
            {/if}
        {/if}
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: #1e293b; border-radius: 10px; }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: #334155; }
</style>
