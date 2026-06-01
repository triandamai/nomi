<script lang="ts">
    import { onMount } from 'svelte';
    import { friendsStore } from '$lib/stores/friends.svelte';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { profileStore } from '$lib/stores/profile.svelte';
    import { chatStore } from '$lib/stores/chat.svelte';
    import { chatApi } from '$lib/api/client';
    import { 
        Users, UserPlus, Check, X, Search, Loader2, MessageSquare, 
        DollarSign, ShieldAlert, ArrowRight, Hourglass, CheckCircle 
    } from 'lucide-svelte';

    let activeTab = $state<'list' | 'requests' | 'add'>('list');
    let searchVal = $state('');
    let searchResults = $state<any[]>([]);
    let searching = $state(false);
    let errorMsg = $state('');
    let successMsg = $state('');

    onMount(() => {
        friendsStore.fetchFriends();
        friendsStore.fetchRequests();
    });

    async function handleSearch() {
        if (searchVal.trim().length < 2) {
            searchResults = [];
            return;
        }
        searching = true;
        errorMsg = '';
        try {
            const res = await chatApi.searchUsers(searchVal);
            searchResults = res.data || [];
        } catch (e) {
            console.error(e);
        } finally {
            searching = false;
        }
    }

    async function sendRequest(receiverId: string) {
        errorMsg = '';
        successMsg = '';
        try {
            await friendsStore.sendRequest(receiverId);
            successMsg = 'Friend request dispatched!';
            searchResults = searchResults.filter(u => u.id !== receiverId);
        } catch (e: any) {
            errorMsg = e.message || 'Failed to dispatch request';
        }
    }

    async function acceptRequest(senderId: string) {
        try {
            const convId = await friendsStore.respondRequest(senderId, true);
            successMsg = 'Friend request accepted!';
            if (convId) {
                // Instantly sync conversation list
                await profileStore.fetchUserConnections();
            }
        } catch (e: any) {
            errorMsg = e.message || 'Failed to accept request';
        }
    }

    async function declineRequest(senderId: string) {
        try {
            await friendsStore.respondRequest(senderId, false);
            successMsg = 'Friend request declined.';
        } catch (e: any) {
            errorMsg = e.message || 'Failed to decline request';
        }
    }

    async function startDM(friend: any) {
        // Find existing private conversation with this user
        const dm = profileStore.userConversations.find(
            c => c.conversation_type === 'private' && c.title.includes(friend.display_name || friend.name)
        );

        if (dm) {
            chatStore.selectConversation(dm.id);
            popupStore.closeLast();
        } else {
            // Refetch connections to locate the automatically provisioned DM channel
            await profileStore.fetchUserConnections();
            const freshDM = profileStore.userConversations.find(
                c => c.conversation_type === 'private' && c.title.includes(friend.display_name || friend.name)
            );
            if (freshDM) {
                chatStore.selectConversation(freshDM.id);
                popupStore.closeLast();
            } else {
                errorMsg = 'DM channel still establishing. Please refresh.';
            }
        }
    }
</script>

<div class="space-y-6 text-slate-200 bg-transparent">
    <!-- Header Navigation Tabs -->
    <div class="flex items-center justify-between border-b border-slate-800/40 pb-4">
        <span class="text-[10px] font-black uppercase tracking-widest text-slate-500">Manage Connections</span>
        <div class="flex bg-slate-900/60 p-1 rounded-xl border border-slate-800/45">
            <button 
                onclick={() => activeTab = 'list'} 
                class="px-4 py-1.5 rounded-lg text-[9px] font-black uppercase tracking-wider transition-all {activeTab === 'list' ? 'bg-purple-600 text-white shadow-lg shadow-purple-500/20' : 'text-slate-400 hover:text-slate-200'}"
            >
                Contacts
            </button>
            <button 
                onclick={() => activeTab = 'requests'} 
                class="relative px-4 py-1.5 rounded-lg text-[9px] font-black uppercase tracking-wider transition-all {activeTab === 'requests' ? 'bg-purple-600 text-white shadow-lg shadow-purple-500/20' : 'text-slate-400 hover:text-slate-200'}"
            >
                Requests
                {#if friendsStore.incomingRequests.length > 0}
                    <span class="absolute -top-1 -right-1 w-4 h-4 bg-rose-500 text-white font-mono text-[8px] font-black flex items-center justify-center rounded-full animate-bounce">
                        {friendsStore.incomingRequests.length}
                    </span>
                {/if}
            </button>
            <button 
                onclick={() => activeTab = 'add'} 
                class="px-4 py-1.5 rounded-lg text-[9px] font-black uppercase tracking-wider transition-all {activeTab === 'add' ? 'bg-purple-600 text-white shadow-lg shadow-purple-500/20' : 'text-slate-400 hover:text-slate-200'}"
            >
                Find friends
            </button>
        </div>
    </div>

    <!-- Main Dynamic Workspace View -->
    <div class="space-y-4">
        {#if errorMsg}
            <div class="flex items-center gap-2.5 p-3.5 bg-rose-500/10 border border-rose-500/20 rounded-xl text-xs text-rose-400">
                <ShieldAlert size={14} />
                <span>{errorMsg}</span>
            </div>
        {/if}
        {#if successMsg}
            <div class="flex items-center gap-2.5 p-3.5 bg-emerald-500/10 border border-emerald-500/20 rounded-xl text-xs text-emerald-400">
                <CheckCircle size={14} />
                <span>{successMsg}</span>
            </div>
        {/if}

        <!-- Tab: Friends List -->
        {#if activeTab === 'list'}
            {#if friendsStore.loading}
                <div class="flex flex-col items-center justify-center py-24 gap-3">
                    <Loader2 size={24} class="animate-spin text-purple-400" />
                    <span class="text-[9px] font-black uppercase tracking-widest text-slate-500">Syncing Address Book...</span>
                </div>
            {:else if friendsStore.friends.length === 0}
                <div class="flex flex-col items-center justify-center py-20 text-center bg-slate-950/20 border border-dashed border-slate-800 rounded-3xl p-6">
                    <Users size={32} class="text-slate-700 mb-4" />
                    <p class="text-xs font-bold text-slate-400">Address Book Empty</p>
                    <p class="text-[10px] text-slate-500 mt-1 max-w-[200px]">Send a friend request to get connected with peers!</p>
                </div>
            {:else}
                <div class="grid gap-3">
                    {#each friendsStore.friends as f (f.id)}
                        <div class="flex items-center justify-between p-4 bg-slate-900/40 border border-slate-800/40 rounded-2xl hover:border-slate-700/60 transition-all group">
                            <div class="flex items-center gap-3.5 min-w-0">
                                <div class="relative shrink-0 w-10 h-10 rounded-xl bg-slate-800 border border-slate-700 flex items-center justify-center font-bold text-purple-300">
                                    {(f.display_name || f.name || '?').charAt(0).toUpperCase()}
                                    <div class="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-emerald-500 border-2 border-[#0f172a] rounded-full"></div>
                                </div>
                                <div class="min-w-0">
                                    <h4 class="text-xs font-bold text-slate-200 truncate">{f.display_name || f.name}</h4>
                                    <p class="text-[9px] text-slate-500 font-mono truncate mt-0.5">{f.email || 'No email'}</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-2">
                                <button 
                                    onclick={() => startDM(f)}
                                    class="p-2.5 bg-purple-500/10 hover:bg-purple-500/20 border border-purple-500/20 rounded-xl text-purple-400 transition-all hover:scale-105 active:scale-95"
                                    title="Open Chat"
                                >
                                    <MessageSquare size={14} />
                                </button>
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}

        <!-- Tab: Pending Requests -->
        {:else if activeTab === 'requests'}
            <!-- Incoming Requests -->
            <div class="space-y-3.5">
                <h3 class="text-[9px] font-black uppercase text-slate-500 tracking-widest px-1">Incoming Invitations</h3>
                {#if friendsStore.incomingRequests.length === 0}
                    <div class="p-6 text-center bg-slate-950/20 border border-dashed border-slate-800 rounded-2xl text-[10px] text-slate-500 italic">
                        No pending incoming requests
                    </div>
                {:else}
                    <div class="grid gap-2.5">
                        {#each friendsStore.incomingRequests as r (r.id)}
                            <div class="flex items-center justify-between p-3.5 bg-slate-900/40 border border-slate-800/40 rounded-2xl">
                                <div class="min-w-0">
                                    <span class="text-xs font-bold text-slate-200">{r.sender_display_name}</span>
                                    <span class="text-[9px] text-slate-500 block font-mono mt-0.5">Wants to be friends</span>
                                </div>
                                <div class="flex items-center gap-1.5">
                                    <button 
                                        onclick={() => acceptRequest(r.sender_id)}
                                        class="p-2 bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/25 rounded-xl text-emerald-400 transition-all"
                                    >
                                        <Check size={14} />
                                    </button>
                                    <button 
                                        onclick={() => declineRequest(r.sender_id)}
                                        class="p-2 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/25 rounded-xl text-rose-400 transition-all"
                                    >
                                        <X size={14} />
                                    </button>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>

            <!-- Outgoing Requests -->
            <div class="space-y-3.5 pt-4">
                <h3 class="text-[9px] font-black uppercase text-slate-500 tracking-widest px-1">Sent Invitations</h3>
                {#if friendsStore.outgoingRequests.length === 0}
                    <div class="p-6 text-center bg-slate-950/20 border border-dashed border-slate-800 rounded-2xl text-[10px] text-slate-500 italic">
                        No pending sent requests
                    </div>
                {:else}
                    <div class="grid gap-2.5">
                        {#each friendsStore.outgoingRequests as r (r.id)}
                            <div class="flex items-center justify-between p-3.5 bg-slate-900/40 border border-slate-800/40 rounded-2xl">
                                <span class="text-xs font-bold text-slate-400">{r.receiver_display_name}</span>
                                <div class="flex items-center gap-1.5 text-[9px] text-slate-600 font-mono">
                                    <Hourglass size={10} />
                                    <span>Awaiting Action</span>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>

        <!-- Tab: Add/Find Friends -->
        {:else if activeTab === 'add'}
            <div class="space-y-4">
                <div class="relative">
                    <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
                    <input 
                        type="text" 
                        placeholder="Search users by name or email..." 
                        bind:value={searchVal}
                        oninput={handleSearch}
                        class="w-full bg-slate-900 border border-slate-800 rounded-2xl pl-11 pr-4 py-3.5 text-xs focus:outline-none focus:ring-2 focus:ring-purple-500/50 transition-all placeholder:text-slate-600"
                    />
                </div>

                <div class="grid gap-2.5 pt-2">
                    {#if searching}
                        <div class="flex items-center justify-center py-12 gap-2">
                            <Loader2 size={16} class="animate-spin text-purple-400" />
                            <span class="text-[9px] font-black uppercase tracking-wider text-slate-500">Searching Registry...</span>
                        </div>
                    {:else if searchResults.length === 0}
                        {#if searchVal.trim().length >= 2}
                            <p class="text-center text-xs text-slate-600 italic py-12">No users found matching "{searchVal}"</p>
                        {/if}
                    {:else}
                        {#each searchResults as u (u.id)}
                            <div class="flex items-center justify-between p-4 bg-slate-900/40 border border-slate-800/40 rounded-2xl hover:border-slate-700/60 transition-all">
                                <div>
                                    <h4 class="text-xs font-bold text-slate-200">{u.display_name || u.name}</h4>
                                    <p class="text-[9px] text-slate-500 font-mono mt-0.5">{u.email || 'No email'}</p>
                                </div>
                                <button 
                                    onclick={() => sendRequest(u.id)}
                                    class="flex items-center gap-1.5 px-4 py-2 bg-purple-600 hover:bg-purple-500 text-white rounded-xl text-[10px] font-black uppercase tracking-wider transition-all active:scale-95 shadow-md shadow-purple-500/10"
                                >
                                    <UserPlus size={12} />
                                    Invite
                                </button>
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>
        {/if}
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: #1e293b; border-radius: 10px; }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: #334155; }
</style>
