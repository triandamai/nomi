<script lang="ts">
    import { onMount } from 'svelte';
    import { adminStore } from '$lib/stores/admin.svelte';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { ChevronDown, Loader2, User, Search, Calendar, ShieldCheck, Mail, Fingerprint, Database, X, Save, Link, MessageSquare } from 'lucide-svelte';

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

    async function openDetail(user: any) {
        await adminStore.fetchUserDetail(user.id);
        popupStore.open({
            title: 'User Profile & Connections',
            width: 'max-w-2xl',
            contentSnippet: detailSnippet,
            footerSnippet: detailFooter
        });
    }

    let isUpdating = $state(false);
    let editRole = $state('');
    let editVerified = $state(false);

    $effect(() => {
        if (adminStore.selectedUserDetail) {
            editRole = adminStore.selectedUserDetail.user.role;
            editVerified = adminStore.selectedUserDetail.user.is_verified || false;
        }
    });

    async function handleUpdate() {
        if (!adminStore.selectedUserDetail) return;
        isUpdating = true;
        try {
            await adminStore.updateAdminUser(adminStore.selectedUserDetail.user.id, {
                role: editRole,
                is_verified: editVerified
            });
            popupStore.closeLast();
        } catch (e) {
            console.error(e);
        } finally {
            isUpdating = false;
        }
    }

    async function handleDelete() {
        if (!adminStore.selectedUserDetail) return;
        if (!confirm('Are you sure you want to delete this user? This action is permanent.')) return;
        
        isUpdating = true;
        try {
            await adminStore.deleteAdminUser(adminStore.selectedUserDetail.user.id);
            popupStore.closeLast();
        } catch (e) {
            console.error(e);
        } finally {
            isUpdating = false;
        }
    }
</script>

{#snippet detailSnippet()}
    {#if adminStore.selectedUserDetail}
        {@const u = adminStore.selectedUserDetail.user}
        <div class="space-y-8 py-2">
            <!-- User Info Section -->
            <div class="space-y-4">
                <div class="flex items-center gap-4">
                    <div class="w-16 h-16 rounded-3xl bg-slate-800 border border-slate-700 flex items-center justify-center text-2xl font-black text-purple-400 shadow-xl shrink-0">
                        {(u.display_name || u.name || '?').charAt(0).toUpperCase()}
                    </div>
                    <div class="min-w-0">
                        <h3 class="text-xl font-black text-white truncate leading-none">{u.display_name || 'Anonymous'}</h3>
                        <p class="text-xs text-slate-500 font-mono mt-2 truncate">UUID: {u.id}</p>
                    </div>
                </div>

                <div class="bg-slate-950/50 border border-slate-800 rounded-2xl p-4 space-y-4">
                    <!-- Identity Metadata -->
                    <div class="space-y-3 pb-4 border-b border-slate-800/50">
                        <div class="flex items-center gap-3 text-xs">
                            <Mail size={14} class="text-slate-600" />
                            <span class="text-slate-300 font-medium">{u.email || 'No email associated'}</span>
                        </div>
                        <div class="flex items-center gap-3 text-xs">
                            <Calendar size={14} class="text-slate-600" />
                            <span class="text-slate-300 font-medium">Account created {formatDate(u.created_at)}</span>
                        </div>
                    </div>

                    <!-- Access Management -->
                    <div class="space-y-4">
                        <div class="space-y-2">
                            <p class="text-[9px] font-black uppercase text-slate-600 tracking-widest ml-1">Assigned Role</p>
                            <select 
                                bind:value={editRole}
                                class="w-full bg-slate-900 border border-slate-800 rounded-xl px-4 py-2.5 text-xs font-bold text-slate-200 focus:outline-none focus:ring-2 focus:ring-purple-500/50"
                            >
                                <option value="user">USER</option>
                                <option value="admin">ADMIN</option>
                                <option value="moderator">MODERATOR</option>
                            </select>
                        </div>

                        <div class="flex items-center justify-between p-3.5 bg-slate-900/50 border border-slate-800 rounded-xl">
                            <div>
                                <p class="text-xs font-bold text-slate-200">Verified Account</p>
                                <p class="text-[9px] text-slate-500 mt-0.5">Grants elevated trust clearance</p>
                            </div>
                            <button 
                                onclick={() => editVerified = !editVerified}
                                class="w-10 h-5 rounded-full transition-all relative {editVerified ? 'bg-emerald-600' : 'bg-slate-800'}"
                                aria-label="Toggle Verification"
                            >
                                <div class="absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-all {editVerified ? 'translate-x-5' : ''}"></div>
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Connected Channels Section -->
            <div class="space-y-4">
                <div class="flex items-center gap-2 px-1">
                    <Link size={16} class="text-slate-500" />
                    <h4 class="text-[10px] font-black uppercase tracking-widest text-slate-500">Connected Channels</h4>
                </div>

                <div class="grid gap-2">
                    {#if adminStore.selectedUserDetail.channels.length === 0}
                        <div class="p-8 text-center bg-slate-950/30 border border-dashed border-slate-800 rounded-2xl">
                            <p class="text-xs text-slate-600 italic">No external messaging channels connected.</p>
                        </div>
                    {:else}
                        {#each adminStore.selectedUserDetail.channels as ch}
                            <div class="p-4 bg-slate-950 border border-slate-800 rounded-2xl group hover:border-slate-700 transition-colors">
                                <div class="flex items-center gap-4">
                                    <div class="w-10 h-10 rounded-xl bg-slate-900 border border-slate-800 flex items-center justify-center text-slate-400">
                                        {#if ch.channel_type === 'whatsapp'}
                                            <svg viewBox="0 0 24 24" class="w-4 h-4 fill-current"><path d="M12.04 2c-5.46 0-9.91 4.45-9.91 9.91 0 1.75.46 3.45 1.32 4.95L2.05 22l5.25-1.38c1.45.79 3.08 1.21 4.74 1.21 5.46 0 9.91-4.45 9.91-9.91 0-2.65-1.03-5.14-2.9-7.01A9.817 9.817 0 0012.04 2m.01 1.67c2.2 0 4.26.86 5.82 2.42 1.56 1.56 2.41 3.63 2.41 5.83 0 4.54-3.7 8.23-8.24 8.23-1.48 0-2.93-.39-4.19-1.15l-.3-.17-3.12.82.83-3.04-.19-.3a8.132 8.132 0 01-1.26-4.38c.01-4.54 3.7-8.24 8.24-8.24m-3.53 4.75c-.19 0-.52.07-.79.37-.27.3-.87.85-.87 2.08s.89 2.42 1.01 2.58c.12.16 1.75 2.67 4.23 3.74.59.26 1.05.41 1.41.52.59.19 1.13.16 1.56.1.48-.07 1.47-.6 1.67-1.18.21-.58.21-1.07.14-1.18-.06-.1-.23-.16-.48-.27-.25-.12-1.47-.73-1.69-.82-.23-.09-.39-.12-.56.12-.17.25-.64.81-.78.97-.14.17-.29.19-.54.06-.25-.12-1.05-.39-1.99-1.23-.74-.66-1.23-1.47-1.38-1.72-.14-.25-.01-.39.11-.51.11-.11.25-.29.37-.43.12-.14.17-.25.25-.41.08-.16.04-.31-.02-.43-.06-.12-.56-1.35-.77-1.85-.2-.5-.4-.43-.56-.44l-.48-.01z"/></svg>
                                        {:else}
                                            <Mail size={18} />
                                        {/if}
                                    </div>
                                    <div class="min-w-0">
                                        <p class="text-xs font-black text-white uppercase tracking-widest">{ch.channel_type}</p>
                                        <p class="text-[10px] text-slate-500 font-mono truncate">{ch.external_id}</p>
                                    </div>
                                </div>
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>

            <!-- Connected Conversations Section -->
            <div class="space-y-4">
                <div class="flex items-center gap-2 px-1">
                    <MessageSquare size={16} class="text-slate-500" />
                    <h4 class="text-[10px] font-black uppercase tracking-widest text-slate-500">Connected Conversations</h4>
                </div>

                <div class="grid gap-2">
                    {#if adminStore.selectedUserDetail.conversations.length === 0}
                        <div class="p-8 text-center bg-slate-950/30 border border-dashed border-slate-800 rounded-2xl">
                            <p class="text-xs text-slate-600 italic">No direct conversation memberships found.</p>
                        </div>
                    {:else}
                        {#each adminStore.selectedUserDetail.conversations as conv}
                            <div class="p-4 bg-slate-950 border border-slate-800 rounded-2xl group hover:border-slate-700 transition-colors">
                                <p class="text-xs font-bold text-slate-200 truncate">{conv.title || 'Private Intelligent Sandbox'}</p>
                                <div class="flex items-center gap-1.5 mt-1.5 text-[9px] text-slate-600 font-mono">
                                    <Calendar size={10} />
                                    <span>Joined: {formatDate(conv.joined_at)}</span>
                                </div>
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>
        </div>
    {:else}
        <div class="flex flex-col items-center justify-center py-32 gap-4">
            <Loader2 size={32} class="animate-spin text-purple-500" />
            <p class="text-xs font-black uppercase tracking-widest text-slate-600">Reconstructing Profile...</p>
        </div>
    {/if}
{/snippet}

{#snippet detailFooter()}
    <div class="flex items-center justify-between gap-3 w-full">
        <button 
            onclick={handleDelete}
            disabled={isUpdating}
            class="flex items-center gap-2 px-6 py-2.5 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 rounded-xl text-rose-500 text-[10px] font-black uppercase tracking-widest transition-all active:scale-95 disabled:opacity-50"
        >
            <X size={14} />
            Erase User
        </button>
        
        <div class="flex items-center gap-3">
            <button 
                onclick={() => popupStore.closeLast()} 
                class="px-6 py-2.5 text-xs font-bold text-slate-500 hover:text-slate-200 transition-colors"
            >
                Discard
            </button>
            <button 
                onclick={handleUpdate} 
                disabled={isUpdating}
                class="flex items-center gap-2 px-8 py-2.5 bg-purple-600 hover:bg-purple-500 disabled:opacity-50 rounded-xl text-white text-xs font-black uppercase tracking-widest shadow-lg shadow-purple-500/20 transition-all active:scale-95"
            >
                {#if isUpdating} <Loader2 size={14} class="animate-spin" /> Synchronizing... {:else} <Save size={14} /> Commit Changes {/if}
            </button>
        </div>
    </div>
{/snippet}

<div class="space-y-4 text-slate-200 bg-transparent">
    <div class="sticky top-0 bg-[#0f172a]/95 backdrop-blur-md border-b border-slate-800/60 p-4 -mx-6 -mt-6 z-10 space-y-4">
        <div class="relative">
            <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
            <input 
                type="text" 
                placeholder="Search by name, email..." 
                oninput={handleSearch}
                value={adminStore.userSearchQuery}
                class="w-full bg-[#04060b] border border-slate-800/80 rounded-xl pl-11 pr-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-purple-500/40 focus:border-purple-500/40 transition-all placeholder:text-slate-600"
            />
        </div>
    </div>

    <!-- User List Body -->
    <div class="space-y-3 pt-2">
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
                    <button 
                        onclick={() => openDetail(user)}
                        class="w-full text-left bg-slate-900/40 border border-slate-800/50 rounded-2xl p-4 md:p-5 hover:border-slate-700 transition-all group active:scale-[0.99]"
                    >
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
                    </button>
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
