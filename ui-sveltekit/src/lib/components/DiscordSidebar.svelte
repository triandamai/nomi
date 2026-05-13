<script lang="ts">
    import { Plus, Edit2, Trash2, Link, Copy, Check, LogOut, User, Settings, Bell, Database, Settings2, RefreshCw, MessageSquare } from 'lucide-svelte';
    import Avatar from './Avatar.svelte';
    import SoulTimeline from './SoulTimeline.svelte';
    import QRCode from './QRCode.svelte';
    import { conversationStore } from '$lib/stores/conversation.svelte';
    import { profileStore } from '$lib/stores/profile.svelte';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { sidebarStore } from '$lib/stores/sidebar.svelte';
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';

    onMount(() => {
        profileStore.fetchProfile();
        sidebarStore.init();
    });

    function handleAddConversation() {
        sidebarStore.newConvName = '';
        popupStore.open({
            title: 'Create Conversation',
            width: 'max-w-md',
            contentSnippet: createConvContent,
            footerSnippet: createConvFooter
        });
    }

    function handleEditConversation(conv: any) {
        sidebarStore.setEditingConv(conv);
        popupStore.open({
            title: 'Edit Conversation',
            width: 'max-w-md',
            contentSnippet: editConvContent,
            footerSnippet: editConvFooter
        });
    }

    function handleDeleteConversation(conv: any) {
        sidebarStore.setEditingConv(conv);
        popupStore.open({
            title: 'Delete Conversation',
            width: 'max-w-sm',
            contentSnippet: deleteConvContent,
            footerSnippet: deleteConvFooter
        });
    }

    async function createConversation() {
        if (sidebarStore.newConvName.trim()) {
            await conversationStore.addConversation(sidebarStore.newConvName.trim());
            sidebarStore.newConvName = '';
            popupStore.closeLast();
        }
    }

    async function updateConversation() {
        if (sidebarStore.editingConv && sidebarStore.newConvName.trim()) {
            await conversationStore.updateConversation(sidebarStore.editingConv.id, sidebarStore.newConvName.trim());
            sidebarStore.setEditingConv(null);
            popupStore.closeLast();
        }
    }

    async function deleteConversation() {
        if (sidebarStore.editingConv) {
            await conversationStore.deleteConversation(sidebarStore.editingConv.id);
            sidebarStore.setEditingConv(null);
            popupStore.closeLast();
        }
    }

    async function handlePairing(conv: any) {
        sidebarStore.setEditingConv(conv);
        try {
            await sidebarStore.getPairingCode(conv.id);
            popupStore.open({
                title: 'Link Telegram',
                width: 'max-w-md',
                contentSnippet: pairingContent,
                footerSnippet: pairingFooter
            });
        } catch (e) {
            console.error(e);
        }
    }

    async function handleShowReminders() {
        sidebarStore.showUserMenu = false;
        await sidebarStore.fetchReminders();
        popupStore.open({
            title: 'Your Reminders',
            width: 'max-w-md',
            contentSnippet: remindersContent,
            footerSnippet: remindersFooter
        });
    }

    function openTimeline() {
        sidebarStore.showUserMenu = false;
        popupStore.open({
            title: 'Soul Timeline',
            width: 'w-full md:w-1/2 lg:w-1/3 xl:w-1/3',
            contentSnippet: soulTimelineSnippet
        });
    }

    function openConnectionManager() {
        sidebarStore.showUserMenu = false;
        popupStore.open({
            title: 'App Connections',
            width: 'max-w-md',
            contentSnippet: connectionManagementSnippet
        });
    }

    function handleLogout() {
        sidebarStore.showUserMenu = false;
        profileStore.logout();
    }
</script>

{#snippet createConvContent()}
    <div class="space-y-4">
        <p class="text-xs text-zinc-500 uppercase font-bold tracking-widest">Conversation Name</p>
        <input
            type="text"
            bind:value={sidebarStore.newConvName}
            placeholder="e.g. general-chat"
            class="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-4 py-3 text-sm text-zinc-200 focus:outline-none focus:ring-1 focus:ring-emerald-500/50 focus:border-emerald-500/50 transition-all"
            onkeydown={(e) => e.key === 'Enter' && createConversation()}
            autofocus
        />
        <p class="text-[10px] text-zinc-600 leading-relaxed">
            By creating a conversation, you can organize your AI interactions into different topics.
        </p>
    </div>
{/snippet}

{#snippet createConvFooter()}
    <div class="flex justify-end gap-3">
        <button
            onclick={() => popupStore.closeLast()}
            class="px-4 py-2 text-xs font-medium text-zinc-400 hover:text-zinc-200"
        >
            Cancel
        </button>
        <button
            onclick={createConversation}
            disabled={!sidebarStore.newConvName.trim()}
            class="px-6 py-2 text-xs font-bold uppercase tracking-wider bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 disabled:hover:bg-emerald-600 rounded-lg text-white transition-all shadow-lg shadow-emerald-900/20"
        >
            Create
        </button>
    </div>
{/snippet}

{#snippet editConvContent()}
    <div class="space-y-4">
        <p class="text-xs text-zinc-500 uppercase font-bold tracking-widest">Edit Name</p>
        <input
            type="text"
            bind:value={sidebarStore.newConvName}
            class="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-4 py-3 text-sm text-zinc-200 focus:outline-none focus:ring-1 focus:ring-emerald-500/50 focus:border-emerald-500/50 transition-all"
            onkeydown={(e) => e.key === 'Enter' && updateConversation()}
            autofocus
        />
    </div>
{/snippet}

{#snippet editConvFooter()}
    <div class="flex justify-end gap-3">
        <button
            onclick={() => popupStore.closeLast()}
            class="px-4 py-2 text-xs font-medium text-zinc-400 hover:text-zinc-200"
        >
            Cancel
        </button>
        <button
            onclick={updateConversation}
            disabled={!sidebarStore.newConvName.trim()}
            class="px-6 py-2 text-xs font-bold uppercase tracking-wider bg-emerald-600 hover:bg-emerald-500 rounded-lg text-white transition-all"
        >
            Save Changes
        </button>
    </div>
{/snippet}

{#snippet deleteConvContent()}
    <div class="space-y-4">
        <p class="text-sm text-zinc-300">
            Are you sure you want to delete <span class="font-bold text-zinc-100">{sidebarStore.editingConv?.name}</span>? This action cannot be undone.
        </p>
    </div>
{/snippet}

{#snippet deleteConvFooter()}
    <div class="flex justify-end gap-3">
        <button
            onclick={() => popupStore.closeLast()}
            class="px-4 py-2 text-xs font-medium text-zinc-400 hover:text-zinc-200"
        >
            Cancel
        </button>
        <button
            onclick={deleteConversation}
            class="px-6 py-2 text-xs font-bold uppercase tracking-wider bg-rose-600 hover:bg-rose-500 rounded-lg text-white transition-all"
        >
            Delete
        </button>
    </div>
{/snippet}

{#snippet remindersContent()}
    <div class="space-y-4">
        {#if sidebarStore.reminders.length === 0}
            <div class="text-center py-8">
                <Bell class="w-12 h-12 text-zinc-800 mx-auto mb-3" />
                <p class="text-sm text-zinc-400">You have no upcoming reminders.</p>
                <p class="text-xs text-zinc-600 mt-1">Ask the AI to set a reminder for you!</p>
            </div>
        {:else}
            <div class="space-y-3">
                {#each sidebarStore.reminders as reminder}
                    <div class="bg-zinc-900/50 border border-zinc-800/50 rounded-xl p-4 transition-all hover:bg-zinc-900">
                        <div class="flex justify-between items-start gap-4">
                            <p class="text-sm text-zinc-200 leading-relaxed whitespace-pre-wrap">{reminder.content}</p>
                            {#if reminder.frequency && reminder.frequency !== 'once'}
                                <span class="shrink-0 text-[10px] px-2 py-1 bg-zinc-800/80 text-zinc-400 rounded-md uppercase font-bold tracking-wider">
                                    {reminder.frequency}
                                </span>
                            {/if}
                        </div>
                        <div class="flex items-center justify-between mt-3 text-xs text-zinc-500">
                            <div class="flex items-center gap-1.5 font-mono bg-black/20 px-2 py-1 rounded">
                                <span class="text-emerald-500/70">
                                    {new Date(reminder.due_at).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}
                                </span>
                                <span>&middot;</span>
                                <span class="text-emerald-500">
                                    {new Date(reminder.due_at).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })}
                                </span>
                            </div>
                            {#if reminder.status === 'completed'}
                                <span class="text-emerald-600 font-medium">Completed</span>
                            {:else if reminder.status === 'archived'}
                                <span class="text-zinc-600 font-medium">Archived</span>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
{/snippet}

{#snippet remindersFooter()}
    <div class="flex justify-end">
        <button
            onclick={() => popupStore.closeLast()}
            class="px-6 py-2 text-xs font-bold uppercase tracking-wider bg-zinc-800 hover:bg-zinc-700 rounded-lg text-white transition-all"
        >
            Close
        </button>
    </div>
{/snippet}

{#snippet pairingContent()}
    <div class="space-y-6 py-2">
        <div class="bg-zinc-950 border border-zinc-800 rounded-xl p-6 flex flex-col items-center gap-4">
            <p class="text-xs text-zinc-500 uppercase font-bold tracking-widest">Internal Pairing Code</p>
            <div class="text-5xl font-black text-emerald-400 tracking-[0.2em] font-mono">
                {sidebarStore.pairingCode}
            </div>
            <button 
                onclick={() => sidebarStore.copyToClipboard()}
                class="flex items-center gap-2 px-4 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 rounded-lg text-xs text-zinc-300 transition-all"
            >
                {#if sidebarStore.copied}
                    <Check size={14} class="text-emerald-400" />
                    <span class="text-emerald-400">Copied!</span>
                {:else}
                    <Copy size={14} />
                    <span>Copy Code</span>
                {/if}
            </button>
        </div>

        <div class="space-y-3">
            <p class="text-xs text-zinc-400 font-bold uppercase tracking-wider">Instructions</p>
            <ol class="text-sm text-zinc-400 space-y-2 list-decimal list-inside">
                {#if sidebarStore.currentPlatform === 'telegram'}
                    <li>Open <a href="https://t.me/ArtaOpenAgentBot" target="_blank" class="text-emerald-400 hover:underline">@ArtaOpenAgentBot</a></li>
                {:else}
                    <li>Open our bot on WhatsApp</li>
                {/if}
                <li>Send the command: <code class="bg-zinc-900 px-1.5 py-0.5 rounded text-emerald-400 font-mono">/pair {sidebarStore.pairingCode}</code></li>
                <li>Wait for confirmation here</li>
            </ol>
        </div>
    </div>
{/snippet}

{#snippet pairingFooter()}
    <div class="flex justify-center w-full">
        <button
            onclick={() => popupStore.closeLast()}
            class="px-8 py-2 text-xs font-bold uppercase tracking-wider text-zinc-500 hover:text-zinc-200 transition-all"
        >
            Cancel
        </button>
    </div>
{/snippet}

{#snippet soulTimelineSnippet()}
    {#if conversationStore.activeConversationId}
        <SoulTimeline conversationId={conversationStore.activeConversationId} />
    {/if}
{/snippet}

{#snippet connectionManagementSnippet()}
    <div class="space-y-4 py-2">
        <div class="flex items-center justify-between px-1">
            <p class="text-xs text-zinc-500 font-medium">Manage your connected messaging platforms.</p>
            {#if profileStore.currentUser?.role === 'admin'}
            <button 
                onclick={() => sidebarStore.openWhatsappBotManager(whatsappBotSetupSnippet, pairingFooter)}
                class="text-[10px] font-black uppercase tracking-tighter text-emerald-500 hover:text-emerald-400 transition-colors bg-emerald-500/5 px-2 py-1 rounded border border-emerald-500/10"
            >
                Bot Setup
            </button>
            {/if}
        </div>
        <div class="grid gap-3">
            {#each sidebarStore.channels as channel}
                <div class="flex items-center justify-between p-4 bg-zinc-950 border border-zinc-800 rounded-xl hover:border-zinc-700 transition-colors">
                    <div class="flex items-center gap-4">
                        <div class="w-10 h-10 rounded-full bg-zinc-900 border border-zinc-800 flex items-center justify-center text-zinc-400">
                            {#if channel.platform === 'telegram'}
                                <svg viewBox="0 0 24 24" class="w-5 h-5 fill-current"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69a.2.2 0 00-.05-.18c-.06-.05-.14-.03-.21-.02-.09.02-1.49.95-4.22 2.79-.4.27-.76.41-1.08.4-.36-.01-1.04-.2-1.55-.37-.63-.2-1.12-.31-1.08-.66.02-.18.27-.36.74-.55 2.92-1.27 4.86-2.11 5.83-2.51 2.78-1.16 3.35-1.36 3.73-1.36.08 0 .27.02.39.12.1.08.13.19.14.27-.01.06.01.24 0 .38z"/></svg>
                            {:else if channel.platform === 'whatsapp'}
                                <svg viewBox="0 0 24 24" class="w-5 h-5 fill-current"><path d="M12.04 2c-5.46 0-9.91 4.45-9.91 9.91 0 1.75.46 3.45 1.32 4.95L2.05 22l5.25-1.38c1.45.79 3.08 1.21 4.74 1.21 5.46 0 9.91-4.45 9.91-9.91 0-2.65-1.03-5.14-2.9-7.01A9.817 9.817 0 0012.04 2m.01 1.67c2.2 0 4.26.86 5.82 2.42 1.56 1.56 2.41 3.63 2.41 5.83 0 4.54-3.7 8.23-8.24 8.23-1.48 0-2.93-.39-4.19-1.15l-.3-.17-3.12.82.83-3.04-.19-.3a8.132 8.132 0 01-1.26-4.38c.01-4.54 3.7-8.24 8.24-8.24m-3.53 4.75c-.19 0-.52.07-.79.37-.27.3-.87.85-.87 2.08s.89 2.42 1.01 2.58c.12.16 1.75 2.67 4.23 3.74.59.26 1.05.41 1.41.52.59.19 1.13.16 1.56.1.48-.07 1.47-.6 1.67-1.18.21-.58.21-1.07.14-1.18-.06-.1-.23-.16-.48-.27-.25-.12-1.47-.73-1.69-.82-.23-.09-.39-.12-.56.12-.17.25-.64.81-.78.97-.14.17-.29.19-.54.06-.25-.12-1.05-.39-1.99-1.23-.74-.66-1.23-1.47-1.38-1.72-.14-.25-.01-.39.11-.51.11-.11.25-.29.37-.43.12-.14.17-.25.25-.41.08-.16.04-.31-.02-.43-.06-.12-.56-1.35-.77-1.85-.2-.5-.4-.43-.56-.44l-.48-.01z"/></svg>
                            {:else}
                                <MessageSquare size={20} />
                            {/if}
                        </div>
                        <div>
                            <p class="text-sm font-bold text-zinc-100 capitalize">{channel.platform}</p>
                            <p class="text-[11px] text-zinc-500 font-medium">
                                {channel.paired ? 'Currently linked' : 'Not connected yet'}
                            </p>
                        </div>
                    </div>
                    
                    {#if channel.paired}
                        <div class="flex items-center gap-1.5 px-3 py-1 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                            <Check size={12} strokeWidth={3} />
                            <span class="text-[10px] font-black uppercase tracking-widest">Linked</span>
                        </div>
                    {:else}
                        <button 
                            onclick={() => sidebarStore.handlePairing(channel.platform, pairingContent, pairingFooter)}
                            class="px-4 py-1.5 rounded-lg bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-300 text-xs font-bold transition-all active:scale-95"
                        >
                            Connect
                        </button>
                    {/if}
                </div>
            {/each}
        </div>
    </div>
{/snippet}

{#snippet whatsappBotSetupSnippet()}
    <div class="space-y-6 py-2">
        <div class="bg-zinc-950 border border-zinc-800 rounded-xl p-8 flex flex-col items-center gap-6">
            <p class="text-xs text-zinc-500 uppercase font-bold tracking-widest">Scan to Connect Bot</p>
            
            <div class="relative group">
                <QRCode data={sidebarStore.whatsappQr} size={220} />
                {#if sidebarStore.isLoadingQr}
                    <div class="absolute inset-0 bg-zinc-950/80 rounded-lg flex items-center justify-center backdrop-blur-sm">
                        <div class="w-8 h-8 border-4 border-zinc-800 border-t-emerald-500 rounded-full animate-spin"></div>
                    </div>
                {/if}
            </div>

            <div class="flex items-center gap-3">
                <button 
                    onclick={() => sidebarStore.fetchWhatsappQr()}
                    disabled={sidebarStore.isLoadingQr}
                    class="flex items-center gap-2 px-4 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 rounded-lg text-xs text-zinc-300 transition-all disabled:opacity-50"
                >
                    <RefreshCw size={14} class={sidebarStore.isLoadingQr ? 'animate-spin' : ''} />
                    <span>Refresh QR</span>
                </button>

                <button 
                    onclick={() => sidebarStore.handleWhatsappLogout()}
                    disabled={sidebarStore.isLoadingQr}
                    class="flex items-center gap-2 px-4 py-2 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 rounded-lg text-xs text-rose-400 transition-all disabled:opacity-50"
                >
                    <RefreshCw size={14} />
                    <span>Logout & Reset</span>
                </button>
            </div>
        </div>

        <div class="space-y-3 px-1">
            <p class="text-xs text-zinc-400 font-bold uppercase tracking-wider">Bot Instructions</p>
            <p class="text-sm text-zinc-500 leading-relaxed">
                Scanning this QR code links your WhatsApp account to our backend service. This allows Arta to send and receive messages as you.
            </p>
            <ol class="text-sm text-zinc-400 space-y-2 list-decimal list-inside mt-2">
                <li>Open WhatsApp on your phone</li>
                <li>Go to <span class="text-zinc-200">Linked Devices</span></li>
                <li>Scan this QR code</li>
            </ol>
        </div>
    </div>
{/snippet}

<aside class="w-[72px] h-screen bg-[#0c0c0e] border-r border-zinc-800 flex flex-col items-center py-3 gap-3">
    <!-- Home / Logo -->
    <div class="mb-2">
        <div class="w-12 h-12 bg-zinc-100 rounded-[16px] flex items-center justify-center cursor-pointer hover:bg-white transition-all">
            <span class="text-zinc-950 font-bold text-xl">O</span>
        </div>
    </div>

    <div class="w-8 h-[2px] bg-zinc-800 rounded-full mb-1"></div>

    <!-- Conversations -->
    <div class="flex-1 pt-2 w-full flex flex-col items-center gap-3 overflow-y-auto no-scrollbar">
        {#each conversationStore.conversations as conv}
            <div class="group relative w-full flex justify-center">
                <Avatar
                    name={conv.name}
                    active={conv.id === conversationStore.activeConversationId}
                    online={conv.online}
                    onClick={() => conversationStore.setActive(conv.id)}
                />

                <!-- Action Tooltip/Menu -->
                <div class="absolute left-16 hidden group-hover:flex bg-zinc-950 border border-zinc-800 rounded-lg p-1 shadow-2xl z-50">
                    <button
                        onclick={() => handlePairing(conv)}
                        class="p-2 hover:bg-zinc-800 text-zinc-400 hover:text-emerald-400 rounded-md transition-colors"
                        title="Link Telegram"
                    >
                        <Link size={14} />
                    </button>
                    <button
                        onclick={() => handleEditConversation(conv)}
                        class="p-2 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 rounded-md transition-colors"
                        title="Edit"
                    >
                        <Edit2 size={14} />
                    </button>
                    <button
                        onclick={() => handleDeleteConversation(conv)}
                        class="p-2 hover:bg-rose-900/30 text-zinc-400 hover:text-rose-400 rounded-md transition-colors"
                        title="Delete"
                    >
                        <Trash2 size={14} />
                    </button>
                </div>
            </div>
        {/each}
    </div>

    <!-- Bottom Actions -->
    <div class="w-full flex flex-col items-center gap-3 mt-auto pt-3 border-t border-zinc-800">
        <!-- Add Button -->
        {#if profileStore.currentUser?.role === 'admin'}
        <button
            onclick={handleAddConversation}
            class="w-12 h-12 rounded-[24px] hover:rounded-[16px] bg-zinc-800 hover:bg-emerald-600 text-emerald-500 hover:text-white flex items-center justify-center transition-all group relative"
        >
            <Plus class="w-6 h-6" />
            <div class="absolute left-16 px-3 py-1 bg-zinc-950 text-white text-xs font-bold rounded shadow-xl whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity z-50">
                Add Conversation
            </div>
        </button>
        {/if}

        <!-- Current User -->
        <div class="relative w-full flex justify-center">
            <Avatar
                name={profileStore.currentUser?.display_name || profileStore.currentUser?.external_id || 'User'}
                active={sidebarStore.showUserMenu}
                online={profileStore.currentUser?.status === 'online'}
                onClick={() => sidebarStore.toggleUserMenu()}
            />

            {#if sidebarStore.showUserMenu}
                <div 
                    class="absolute bottom-0 left-16 w-56 bg-zinc-950 border border-zinc-800 rounded-xl shadow-2xl z-[100] py-2 overflow-hidden animate-in fade-in slide-in-from-left-2 duration-200"
                >
                    <div class="px-4 py-3 border-b border-zinc-900 mb-1">
                        <p class="text-sm font-bold text-zinc-100 truncate">
                            {profileStore.currentUser?.display_name || 'Anonymous User'}
                        </p>
                        <p class="text-[10px] text-zinc-500 truncate font-mono mt-0.5">
                            {profileStore.currentUser?.external_id}
                        </p>
                    </div>

                    <button class="w-full flex items-center gap-3 px-4 py-2 text-xs text-zinc-400 hover:text-zinc-100 hover:bg-zinc-900 transition-colors">
                        <User size={14} />
                        <span>Profile Settings</span>
                    </button>
                    
                    <button class="w-full flex items-center gap-3 px-4 py-2 text-xs text-zinc-400 hover:text-zinc-100 hover:bg-zinc-900 transition-colors">
                        <Settings size={14} />
                        <span>Preferences</span>
                    </button>

                    <div class="h-px bg-zinc-900 my-1"></div>

                    <button 
                        onclick={handleShowReminders}
                        class="w-full flex items-center gap-3 px-4 py-2 text-xs text-zinc-400 hover:text-zinc-100 hover:bg-zinc-900 transition-colors"
                    >
                        <Bell size={14} />
                        <span>Reminders</span>
                    </button>

                    {#if profileStore.currentUser?.role === 'admin'}
                    <button 
                        onclick={() => { sidebarStore.showUserMenu = false; goto('/admin/storage'); }}
                        class="w-full flex items-center gap-3 px-4 py-2 text-xs text-zinc-400 hover:text-zinc-100 hover:bg-zinc-900 transition-colors"
                    >
                        <Database size={14} />
                        <span>Storage</span>
                    </button>
                    {/if}

                    <div class="h-px bg-zinc-900 my-1"></div>

                    {#if conversationStore.activeConversationId}
                    <button 
                        onclick={openConnectionManager}
                        class="w-full flex items-center gap-3 px-4 py-2 text-xs transition-colors {sidebarStore.isPaired ? 'text-emerald-400 hover:text-emerald-300 hover:bg-emerald-900/10' : 'text-zinc-400 hover:text-zinc-100 hover:bg-zinc-900'}"
                    >
                        <Link size={14} />
                        <span>Linked App</span>
                    </button>

                    <button 
                        onclick={openTimeline}
                        class="w-full flex items-center gap-3 px-4 py-2 text-xs text-zinc-400 hover:text-zinc-100 hover:bg-zinc-900 transition-colors"
                    >
                        <Settings2 size={14} />
                        <span>Soul Timeline</span>
                    </button>

                    <div class="h-px bg-zinc-900 my-1"></div>
                    {/if}

                    <button 
                        onclick={handleLogout}
                        class="w-full flex items-center gap-3 px-4 py-2 text-xs text-rose-400 hover:text-rose-300 hover:bg-rose-900/20 transition-colors"
                    >
                        <LogOut size={14} />
                        <span>Sign Out</span>
                    </button>
                </div>

                <!-- Backdrop to close menu -->
                <div 
                    class="fixed inset-0 z-[90]" 
                    onclick={() => sidebarStore.showUserMenu = false}
                    onkeydown={(e) => e.key === 'Escape' && (sidebarStore.showUserMenu = false)}
                    role="button"
                    tabindex="0"
                ></div>
            {/if}
        </div>
    </div>
</aside>

<style>
    .no-scrollbar::-webkit-scrollbar {
        display: none;
    }
    .no-scrollbar {
        -ms-overflow-style: none;
        scrollbar-width: none;
    }
</style>
