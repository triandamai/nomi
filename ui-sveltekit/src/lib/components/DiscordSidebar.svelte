<script lang="ts">
    import {
        Plus,
        Edit2,
        Trash2,
        Link,
        Copy,
        Check,
        LogOut,
        User,
        Settings,
        Bell,
        Database,
        Settings2,
        RefreshCw,
        MessageSquare,
        LineChart,
        DollarSign, Terminal
    } from 'lucide-svelte';
    import Avatar from './Avatar.svelte';
    import SoulTimeline from './SoulTimeline.svelte';
    import QRCode from './QRCode.svelte';
    import TransactionHistoryPopUp from './TransactionHistoryPopUp.svelte';
    import AdminConversationsPopUp from './AdminConversationsPopUp.svelte';
    import UserListPopUp from './UserListPopUp.svelte';
    import RedisTestPopUp from './RedisTestPopUp.svelte';
    import { conversationStore } from '$lib/stores/conversation.svelte';

    import {profileStore} from '$lib/stores/profile.svelte';
    import {popupStore} from '$lib/stores/popup.svelte';
    import {sidebarStore} from '$lib/stores/sidebar.svelte';
    import {onMount} from 'svelte';
    import {goto} from '$app/navigation';

    onMount(() => {
        profileStore.fetchProfile();
        sidebarStore.init();
    });

    function handleAddConversation() {
        sidebarStore.newConvName = '';
        sidebarStore.newConvType = 'private';
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
            await conversationStore.addConversation(sidebarStore.newConvName.trim(), sidebarStore.newConvType);
            sidebarStore.newConvName = '';
            sidebarStore.newConvType = 'private';
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

    function openStockSignals() {
        sidebarStore.showUserMenu = false;
        goto('/admin/stock');
    }

    function openMoneyTracking() {
        sidebarStore.showUserMenu = false;
        popupStore.open({
            title: 'Transaction History',
            width: 'w-full max-w-2xl h-screen',
            contentSnippet: moneyTrackingSnippet
        });
    }

    function openAdminMonitor() {
        sidebarStore.showUserMenu = false;
        popupStore.open({
            title: 'Admin Monitor',
            headerSnippet: adminHeaderSnippet,
            width: 'w-full max-w-2xl h-screen',
            contentSnippet: adminConversationsSnippet
        });
    }

    function openUserDirectory() {
        sidebarStore.showUserMenu = false;
        popupStore.open({
            title: 'User Directory',
            headerSnippet: userHeaderSnippet,
            width: 'w-full max-w-2xl h-screen',
            contentSnippet: userDirectorySnippet
        });
    }

    function openRedisTest() {
        sidebarStore.showUserMenu = false;
        popupStore.open({
            title: 'Redis Pub/Sub Test',
            headerSnippet: redisHeaderSnippet,
            width: 'w-full max-w-md h-screen',
            contentSnippet: redisTestSnippet
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
    <div class="space-y-6">
        <div class="space-y-2">
            <p class="text-[10px] text-slate-500 uppercase font-black tracking-widest">Conversation Name</p>
            <input
                    type="text"
                    bind:value={sidebarStore.newConvName}
                    placeholder="e.g. general-chat"
                    class="w-full bg-slate-950 border border-slate-800 rounded-2xl px-4 py-3 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 transition-all placeholder:text-slate-800"
                    onkeydown={(e) => e.key === 'Enter' && createConversation()}
                    autofocus
            />
        </div>

        <div class="space-y-2">
            <p class="text-[10px] text-slate-500 uppercase font-black tracking-widest">Conversation Type</p>
            <div class="grid grid-cols-2 gap-3">
                <button
                        onclick={() => sidebarStore.newConvType = 'private'}
                        class="flex flex-col items-center gap-3 p-4 rounded-2xl border transition-all {sidebarStore.newConvType === 'private' ? 'bg-blue-500/10 border-blue-500/50 text-blue-400' : 'bg-slate-950 border-slate-800 text-slate-500 hover:border-slate-700'}"
                >
                    <div class="w-10 h-10 rounded-full bg-slate-900 flex items-center justify-center">
                        <User size={20}
                              class={sidebarStore.newConvType === 'private' ? 'text-blue-400' : 'text-slate-600'}/>
                    </div>
                    <div class="text-center">
                        <p class="text-xs font-bold">Private</p>
                        <p class="text-[9px] opacity-60">Single soul session</p>
                    </div>
                </button>

                <button
                        onclick={() => sidebarStore.newConvType = 'group'}
                        class="flex flex-col items-center gap-3 p-4 rounded-2xl border transition-all {sidebarStore.newConvType === 'group' ? 'bg-purple-500/10 border-purple-500/50 text-purple-400' : 'bg-slate-950 border-slate-800 text-slate-500 hover:border-slate-700'}"
                >
                    <div class="w-10 h-10 rounded-full bg-slate-900 flex items-center justify-center">
                        <Database size={20}
                                  class={sidebarStore.newConvType === 'group' ? 'text-purple-400' : 'text-slate-600'}/>
                    </div>
                    <div class="text-center">
                        <p class="text-xs font-bold">Group</p>
                        <p class="text-[9px] opacity-60">Multi-user workspace</p>
                    </div>
                </button>
            </div>
        </div>

        <p class="text-[11px] text-slate-500 leading-relaxed">
            By creating a conversation, you can organize your AI interactions into different intelligent souls.
        </p>
    </div>
{/snippet}

{#snippet createConvFooter()}
    <div class="flex justify-end gap-3">
        <button
                onclick={() => popupStore.closeLast()}
                class="px-4 py-2 text-xs font-bold text-slate-500 hover:text-slate-200 transition-colors"
        >
            Cancel
        </button>
        <button
                onclick={createConversation}
                disabled={!sidebarStore.newConvName.trim()}
                class="px-6 py-2 text-xs font-black uppercase tracking-widest bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:hover:bg-blue-600 rounded-xl text-white transition-all shadow-lg shadow-blue-500/20"
        >
            Create
        </button>
    </div>
{/snippet}

{#snippet editConvContent()}
    <div class="space-y-4">
        <p class="text-[10px] text-slate-500 uppercase font-black tracking-widest">Edit Name</p>
        <input
                type="text"
                bind:value={sidebarStore.newConvName}
                class="w-full bg-slate-950 border border-slate-800 rounded-2xl px-4 py-3 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 transition-all"
                onkeydown={(e) => e.key === 'Enter' && updateConversation()}
                autofocus
        />
    </div>
{/snippet}

{#snippet editConvFooter()}
    <div class="flex justify-end gap-3">
        <button
                onclick={() => popupStore.closeLast()}
                class="px-4 py-2 text-xs font-bold text-slate-500 hover:text-slate-200 transition-colors"
        >
            Cancel
        </button>
        <button
                onclick={updateConversation}
                disabled={!sidebarStore.newConvName.trim()}
                class="px-6 py-2 text-xs font-black uppercase tracking-widest bg-blue-600 hover:bg-blue-500 rounded-xl text-white transition-all"
        >
            Save Changes
        </button>
    </div>
{/snippet}

{#snippet deleteConvContent()}
    <div class="space-y-4">
        <p class="text-sm text-slate-400 leading-relaxed">
            Are you sure you want to delete <span
                class="font-bold text-slate-100">{sidebarStore.editingConv?.name}</span>? This action will permanently
            erase this soul.
        </p>
    </div>
{/snippet}

{#snippet deleteConvFooter()}
    <div class="flex justify-end gap-3">
        <button
                onclick={() => popupStore.closeLast()}
                class="px-4 py-2 text-xs font-bold text-slate-500 hover:text-slate-200 transition-colors"
        >
            Cancel
        </button>
        <button
                onclick={deleteConversation}
                class="px-6 py-2 text-xs font-black uppercase tracking-widest bg-rose-600 hover:bg-rose-500 rounded-xl text-white transition-all shadow-lg shadow-rose-900/20"
        >
            Delete
        </button>
    </div>
{/snippet}

{#snippet remindersContent()}
    <div class="space-y-4">
        {#if sidebarStore.reminders.length === 0}
            <div class="text-center py-8">
                <Bell class="w-12 h-12 text-slate-800 mx-auto mb-3"/>
                <p class="text-sm text-slate-400">You have no upcoming reminders.</p>
                <p class="text-xs text-slate-600 mt-1">Ask Nomi to set a reminder for you!</p>
            </div>
        {:else}
            <div class="space-y-4">
                {#each sidebarStore.reminders as reminder}
                    <div class="bg-slate-900/50 border border-slate-800/50 rounded-2xl p-4 transition-all hover:bg-slate-900 group">
                        <div class="flex items-center gap-2 mb-3">
                            {#if reminder.task_type === 'REMINDER'}
                                <div class="p-1 rounded bg-amber-500/20 text-amber-400">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"
                                         fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
                                         stroke-linejoin="round">
                                        <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/>
                                        <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>
                                    </svg>
                                </div>
                            {:else if reminder.task_type === 'SEND_DM'}
                                <div class="p-1 rounded bg-blue-500/20 text-blue-400">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"
                                         fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
                                         stroke-linejoin="round">
                                        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                                    </svg>
                                </div>
                            {:else if reminder.task_type === 'TRIGGER_AGENT'}
                                <div class="p-1 rounded bg-purple-500/20 text-purple-400">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"
                                         fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
                                         stroke-linejoin="round">
                                        <path d="M12 8V4H8"/>
                                        <rect width="16" height="12" x="4" y="8" rx="2"/>
                                        <path d="M2 14h2"/>
                                        <path d="M20 14h2"/>
                                        <path d="M15 13v2"/>
                                        <path d="M9 13v2"/>
                                    </svg>
                                </div>
                            {/if}
                            <span class="text-[9px] font-black uppercase tracking-widest text-slate-500">{reminder.task_type || 'REMINDER'}</span>

                            {#if reminder.frequency && reminder.frequency !== 'once'}
                                <span class="shrink-0 text-[10px] px-2 py-0.5 bg-slate-800/80 text-slate-400 rounded-md uppercase font-black tracking-widest ml-auto">
                                    {reminder.frequency}
                                </span>
                            {/if}
                        </div>

                        <div class="flex justify-between items-start gap-4 mb-3">
                            <p class="text-sm text-slate-200 font-medium leading-relaxed whitespace-pre-wrap">{reminder.content}</p>
                        </div>

                        <div class="grid grid-cols-2 gap-2 mb-3">
                            <div class="flex flex-col gap-1">
                                <span class="text-[9px] font-black uppercase tracking-widest text-slate-600">User</span>
                                <div class="flex items-center gap-1.5 text-xs text-slate-400">
                                    <User size={10} class="text-slate-500"/>
                                    <span class="truncate">{reminder.user_display_name || 'Anonymous'}</span>
                                </div>
                            </div>
                            <div class="flex flex-col gap-1">
                                <span class="text-[9px] font-black uppercase tracking-widest text-slate-600">Conversation</span>
                                <div class="flex items-center gap-1.5 text-xs text-slate-400">
                                    <MessageSquare size={10} class="text-slate-500"/>
                                    <span class="truncate">{reminder.conversation_title || 'Private Session'}</span>
                                </div>
                            </div>
                        </div>

                        <div class="flex items-center justify-between pt-3 border-t border-slate-800/50 text-xs text-slate-500">
                            <div class="flex items-center gap-1.5 font-mono bg-black/20 px-2 py-1 rounded">
                                <span class="text-blue-500/70">
                                    {new Date(reminder.due_at).toLocaleDateString(undefined, {
                                        month: 'short',
                                        day: 'numeric'
                                    })}
                                </span>
                                <span>&middot;</span>
                                <span class="text-blue-500">
                                    {new Date(reminder.due_at).toLocaleTimeString(undefined, {
                                        hour: '2-digit',
                                        minute: '2-digit'
                                    })}
                                </span>
                                <span class="text-[9px] text-slate-600">WIB</span>
                            </div>
                            {#if reminder.status === 'completed'}
                                <span class="text-emerald-500 font-bold uppercase text-[10px] tracking-widest">Completed</span>
                            {:else if reminder.status === 'archived'}
                                <span class="text-rose-500 font-bold uppercase text-[10px] tracking-widest">Archived</span>
                            {:else}
                                <span class="text-amber-500 font-bold uppercase text-[10px] tracking-widest">Pending</span>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>

            {#if sidebarStore.hasMoreReminders}
                <div class="pt-4 flex justify-center">
                    <button
                            onclick={() => sidebarStore.loadMoreReminders()}
                            disabled={sidebarStore.isLoadingReminders}
                            class="px-6 py-2 text-[10px] font-black uppercase tracking-widest bg-slate-900 border border-slate-800 rounded-xl text-slate-400 hover:text-slate-200 hover:border-slate-700 transition-all disabled:opacity-50"
                    >
                        {#if sidebarStore.isLoadingReminders}
                            <div class="flex items-center gap-2">
                                <div class="w-3 h-3 border-2 border-slate-700 border-t-slate-400 rounded-full animate-spin"></div>
                                Loading...
                            </div>
                        {:else}
                            Load More Reminders
                        {/if}
                    </button>
                </div>
            {/if}
        {/if}
    </div>
{/snippet}

{#snippet remindersFooter()}
    <div class="flex justify-end">
        <button
                onclick={() => popupStore.closeLast()}
                class="px-8 py-2 text-xs font-black uppercase tracking-widest bg-slate-800 hover:bg-slate-700 rounded-xl text-white transition-all border border-slate-700"
        >
            Close
        </button>
    </div>
{/snippet}

{#snippet pairingContent()}
    <div class="space-y-6 py-2">
        <div class="bg-slate-950 border border-slate-800 rounded-3xl p-8 flex flex-col items-center gap-6">
            <p class="text-[10px] text-slate-500 uppercase font-black tracking-[0.2em]">Internal Pairing Code</p>
            <div class="text-5xl font-black text-blue-400 tracking-[0.3em] font-mono">
                {sidebarStore.pairingCode}
            </div>
            <button
                    onclick={() => sidebarStore.copyToClipboard()}
                    class="flex items-center gap-2 px-6 py-2 bg-slate-900 hover:bg-slate-800 border border-slate-800 rounded-xl text-xs text-slate-300 transition-all active:scale-95"
            >
                {#if sidebarStore.copied}
                    <Check size={14} class="text-blue-400"/>
                    <span class="text-blue-400">Copied!</span>
                {:else}
                    <Copy size={14}/>
                    <span>Copy Code</span>
                {/if}
            </button>
        </div>

        <div class="space-y-4">
            <p class="text-[10px] text-slate-500 font-black uppercase tracking-widest">Instructions</p>
            <ol class="text-sm text-slate-400 space-y-3 list-decimal list-inside">
                {#if sidebarStore.currentPlatform === 'telegram'}
                    <li>Open <a href="https://t.me/ArtaOpenAgentBot" target="_blank"
                                class="text-blue-400 hover:underline">@ArtaOpenAgentBot</a></li>
                {:else}
                    <li>Open our bot on WhatsApp</li>
                {/if}
                <li>Send the command: <code
                        class="bg-slate-950 px-2 py-1 rounded text-blue-400 font-mono text-xs border border-slate-800">/pair {sidebarStore.pairingCode}</code>
                </li>
                <li>Wait for automated soul linking</li>
            </ol>
        </div>
    </div>
{/snippet}

{#snippet pairingFooter()}
    <div class="flex justify-center w-full">
        <button
                onclick={() => popupStore.closeLast()}
                class="px-8 py-2 text-xs font-black uppercase tracking-[0.2em] text-slate-600 hover:text-slate-200 transition-all"
        >
            Cancel
        </button>
    </div>
{/snippet}

{#snippet soulTimelineSnippet()}
    {#if conversationStore.activeConversationId}
        <SoulTimeline conversationId={conversationStore.activeConversationId}/>
    {/if}
{/snippet}

{#snippet moneyTrackingSnippet()}
    <TransactionHistoryPopUp/>
{/snippet}

{#snippet adminConversationsSnippet()}
    <AdminConversationsPopUp />
{/snippet}

{#snippet adminHeaderSnippet()}
    <div class="flex items-center gap-3">
        <div class="p-2 bg-blue-500/10 rounded-xl border border-blue-500/20 text-blue-400 shrink-0">
            <Database size={18}/>
        </div>
        <div class="min-w-0">
            <h2 class="text-xs md:text-sm font-black uppercase tracking-widest text-slate-100 truncate">Token
                Monitoring</h2>
            <p class="text-[9px] md:text-[10px] text-slate-500 font-medium truncate">Usage across active souls</p>
        </div>
    </div>
{/snippet}

{#snippet userDirectorySnippet()}
    <UserListPopUp />
{/snippet}

{#snippet userHeaderSnippet()}
    <div class="flex items-center gap-3">
        <div class="p-2 bg-purple-500/10 rounded-xl border border-purple-500/20 text-purple-400 shrink-0">
            <User size={18}/>
        </div>
        <div class="min-w-0">
            <h2 class="text-xs md:text-sm font-black uppercase tracking-widest text-slate-100 truncate">User
                Directory</h2>
            <p class="text-[9px] md:text-[10px] text-slate-500 font-medium truncate">Monitor platform members</p>
        </div>
    </div>
{/snippet}

{#snippet redisTestSnippet()}
    <RedisTestPopUp />
{/snippet}

{#snippet redisHeaderSnippet()}
    <div class="flex items-center gap-3">
        <div class="p-2 bg-amber-500/10 rounded-xl border border-amber-500/20 text-amber-400 shrink-0">
            <Terminal size={18}/>
        </div>
        <div class="min-w-0">
            <h2 class="text-xs md:text-sm font-black uppercase tracking-widest text-slate-100 truncate">Redis
                Test</h2>
            <p class="text-[9px] md:text-[10px] text-slate-500 font-medium truncate">Debug Pub/Sub Events</p>
        </div>
    </div>
{/snippet}

{#snippet connectionManagementSnippet()}

    <div class="space-y-4 py-2">
        <div class="flex items-center justify-between px-1">
            <p class="text-xs text-slate-500 font-medium">Manage your connected messaging platforms.</p>
            {#if profileStore.currentUser?.role === 'admin'}
                <button
                        onclick={() => sidebarStore.openWhatsappBotManager(whatsappBotSetupSnippet, pairingFooter)}
                        class="text-[10px] font-black uppercase tracking-tighter text-blue-500 hover:text-blue-400 transition-colors bg-blue-500/5 px-2 py-1 rounded border border-blue-500/10"
                >
                    Bot Setup
                </button>
            {/if}
        </div>
        <div class="grid gap-3">
            {#each sidebarStore.channels as channel}
                <div class="flex items-center justify-between p-4 bg-slate-950 border border-slate-800 rounded-2xl hover:border-slate-700 transition-colors">
                    <div class="flex items-center gap-4">
                        <div class="w-10 h-10 rounded-full bg-slate-900 border border-slate-800 flex items-center justify-center text-slate-400">
                            {#if channel.platform === 'telegram'}
                                <svg viewBox="0 0 24 24" class="w-5 h-5 fill-current">
                                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69a.2.2 0 00-.05-.18c-.06-.05-.14-.03-.21-.02-.09.02-1.49.95-4.22 2.79-.4.27-.76.41-1.08.4-.36-.01-1.04-.2-1.55-.37-.63-.2-1.12-.31-1.08-.66.02-.18.27-.36.74-.55 2.92-1.27 4.86-2.11 5.83-2.51 2.78-1.16 3.35-1.36 3.73-1.36.08 0 .27.02.39.12.1.08.13.19.14.27-.01.06.01.24 0 .38z"/>
                                </svg>
                            {:else if channel.platform === 'whatsapp'}
                                <svg viewBox="0 0 24 24" class="w-5 h-5 fill-current">
                                    <path d="M12.04 2c-5.46 0-9.91 4.45-9.91 9.91 0 1.75.46 3.45 1.32 4.95L2.05 22l5.25-1.38c1.45.79 3.08 1.21 4.74 1.21 5.46 0 9.91-4.45 9.91-9.91 0-2.65-1.03-5.14-2.9-7.01A9.817 9.817 0 0012.04 2m.01 1.67c2.2 0 4.26.86 5.82 2.42 1.56 1.56 2.41 3.63 2.41 5.83 0 4.54-3.7 8.23-8.24 8.23-1.48 0-2.93-.39-4.19-1.15l-.3-.17-3.12.82.83-3.04-.19-.3a8.132 8.132 0 01-1.26-4.38c.01-4.54 3.7-8.24 8.24-8.24m-3.53 4.75c-.19 0-.52.07-.79.37-.27.3-.87.85-.87 2.08s.89 2.42 1.01 2.58c.12.16 1.75 2.67 4.23 3.74.59.26 1.05.41 1.41.52.59.19 1.13.16 1.56.1.48-.07 1.47-.6 1.67-1.18.21-.58.21-1.07.14-1.18-.06-.1-.23-.16-.48-.27-.25-.12-1.47-.73-1.69-.82-.23-.09-.39-.12-.56.12-.17.25-.64.81-.78.97-.14.17-.29.19-.54.06-.25-.12-1.05-.39-1.99-1.23-.74-.66-1.23-1.47-1.38-1.72-.14-.25-.01-.39.11-.51.11-.11.25-.29.37-.43.12-.14.17-.25.25-.41.08-.16.04-.31-.02-.43-.06-.12-.56-1.35-.77-1.85-.2-.5-.4-.43-.56-.44l-.48-.01z"/>
                                </svg>
                            {:else}
                                <MessageSquare size={20}/>
                            {/if}
                        </div>
                        <div>
                            <p class="text-sm font-bold text-slate-100 capitalize">{channel.platform}</p>
                            <p class="text-[11px] text-slate-500 font-medium">
                                {channel.paired ? 'Currently linked' : 'Not connected yet'}
                            </p>
                        </div>
                    </div>

                    {#if channel.paired}
                        <div class="flex items-center gap-1.5 px-3 py-1 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20">
                            <Check size={12} strokeWidth={3}/>
                            <span class="text-[10px] font-black uppercase tracking-widest">Linked</span>
                        </div>
                    {:else}
                        <button
                                onclick={() => sidebarStore.handlePairing(channel.platform, pairingContent, pairingFooter)}
                                class="px-4 py-1.5 rounded-xl bg-slate-900 hover:bg-slate-800 border border-slate-800 text-slate-300 text-xs font-bold transition-all active:scale-95"
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
        <div class="bg-slate-950 border border-slate-800 rounded-3xl p-8 flex flex-col items-center gap-6">
            <p class="text-[10px] text-slate-500 uppercase font-black tracking-widest">Scan to Connect Bot</p>

            <div class="relative group">
                <QRCode data={sidebarStore.whatsappQr} size={220}/>
                {#if sidebarStore.isLoadingQr}
                    <div class="absolute inset-0 bg-slate-950/80 rounded-3xl flex items-center justify-center backdrop-blur-sm">
                        <div class="w-8 h-8 border-4 border-slate-800 border-t-blue-500 rounded-full animate-spin"></div>
                    </div>
                {/if}
            </div>

            <div class="flex items-center gap-3">
                <button
                        onclick={() => sidebarStore.fetchWhatsappQr()}
                        disabled={sidebarStore.isLoadingQr}
                        class="flex items-center gap-2 px-6 py-2 bg-slate-900 hover:bg-slate-800 border border-slate-800 rounded-xl text-xs text-slate-300 transition-all disabled:opacity-50"
                >
                    <RefreshCw size={14} class={sidebarStore.isLoadingQr ? 'animate-spin' : ''}/>
                    <span>Refresh QR</span>
                </button>

                <button
                        onclick={() => sidebarStore.handleWhatsappLogout()}
                        disabled={sidebarStore.isLoadingQr}
                        class="flex items-center gap-2 px-6 py-2 bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/20 rounded-xl text-xs text-rose-400 transition-all disabled:opacity-50"
                >
                    <RefreshCw size={14}/>
                    <span>Logout & Reset</span>
                </button>
            </div>
        </div>

        <div class="space-y-3 px-1">
            <p class="text-[10px] text-slate-500 font-black uppercase tracking-widest">Bot Instructions</p>
            <p class="text-sm text-slate-500 leading-relaxed">
                Scanning this QR code links your WhatsApp account to our backend service. This allows Nomi to send and
                receive messages as you.
            </p>
            <ol class="text-sm text-slate-400 space-y-2 list-decimal list-inside mt-2">
                <li>Open WhatsApp on your phone</li>
                <li>Go to <span class="text-slate-200">Linked Devices</span></li>
                <li>Scan this QR code</li>
            </ol>
        </div>
    </div>
{/snippet}

<aside class="w-[72px] h-screen bg-[#0f172a] border-r border-slate-800 flex flex-col items-center py-3 gap-3">
    <!-- Home / Logo -->
    <div class="mb-2">
        <a href="/"
           class="w-12 h-12 bg-blue-600 rounded-[16px] flex items-center justify-center cursor-pointer hover:bg-blue-500 transition-all shadow-lg shadow-blue-500/20">
            <span class="text-white font-black text-xl">N</span>
        </a>
    </div>

    <div class="w-8 h-[2px] bg-slate-800 rounded-full mb-1"></div>

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
                <div class="absolute left-16 hidden group-hover:flex bg-slate-950 border border-slate-800 rounded-lg p-1 shadow-2xl z-50">
                    <button
                            onclick={() => handlePairing(conv)}
                            class="p-2 hover:bg-slate-800 text-slate-400 hover:text-blue-400 rounded-md transition-colors"
                            title="Link Telegram"
                    >
                        <Link size={14}/>
                    </button>
                    <button
                            onclick={() => handleEditConversation(conv)}
                            class="p-2 hover:bg-slate-800 text-slate-400 hover:text-slate-200 rounded-md transition-colors"
                            title="Edit"
                    >
                        <Edit2 size={14}/>
                    </button>
                    <button
                            onclick={() => handleDeleteConversation(conv)}
                            class="p-2 hover:bg-rose-900/30 text-slate-400 hover:text-rose-400 rounded-md transition-colors"
                            title="Delete"
                    >
                        <Trash2 size={14}/>
                    </button>
                </div>
            </div>
        {/each}
    </div>

    <!-- Bottom Actions -->
    <div class="w-full flex flex-col items-center gap-3 mt-auto pt-3 border-t border-slate-800">
        <!-- Add Button -->
        {#if profileStore.currentUser?.role === 'admin'}
            <button
                    onclick={handleAddConversation}
                    class="w-12 h-12 rounded-[24px] hover:rounded-[16px] bg-slate-800 hover:bg-blue-600 text-blue-500 hover:text-white flex items-center justify-center transition-all group relative"
            >
                <Plus class="w-6 h-6"/>
                <div class="absolute left-16 px-3 py-1 bg-slate-950 text-white text-xs font-bold rounded shadow-xl whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity z-50">
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
                        class="absolute bottom-0 left-16 w-56 bg-slate-950 border border-slate-800 rounded-xl shadow-2xl z-[100] py-2 overflow-hidden animate-in fade-in slide-in-from-left-2 duration-200"
                >
                    <div class="px-4 py-3 border-b border-slate-900 mb-1">
                        <p class="text-sm font-bold text-slate-100 truncate">
                            {profileStore.currentUser?.display_name || 'Anonymous User'}
                        </p>
                        <p class="text-[10px] text-slate-500 truncate font-mono mt-0.5">
                            {profileStore.currentUser?.external_id}
                        </p>
                    </div>

                    <button class="w-full flex items-center gap-3 px-4 py-2 text-xs text-slate-400 hover:text-slate-100 hover:bg-slate-900 transition-colors">
                        <User size={14}/>
                        <span>Profile Settings</span>
                    </button>

                    <button class="w-full flex items-center gap-3 px-4 py-2 text-xs text-slate-400 hover:text-slate-100 hover:bg-slate-900 transition-colors">
                        <Settings size={14}/>
                        <span>Preferences</span>
                    </button>

                    <div class="h-px bg-slate-900 my-1"></div>

                    <button
                            onclick={handleShowReminders}
                            class="w-full flex items-center gap-3 px-4 py-2 text-xs text-slate-400 hover:text-slate-100 hover:bg-slate-900 transition-colors"
                    >
                        <Bell size={14}/>
                        <span>Reminders</span>
                    </button>
                    <button
                            onclick={openMoneyTracking}
                            class="w-full flex items-center gap-3 px-4 py-2 text-xs text-slate-400 hover:text-slate-100 hover:bg-slate-900 transition-colors"
                    >
                        <DollarSign size={14}/>
                        <span>Money Tracking</span>
                    </button>
                    {#if profileStore.currentUser?.role === 'admin'}
                        <button
                                onclick={() => { sidebarStore.showUserMenu = false; goto('/admin/storage'); }}
                                class="w-full flex items-center gap-3 px-4 py-2 text-xs text-slate-400 hover:text-slate-100 hover:bg-slate-900 transition-colors"
                        >
                            <Database size={14}/>
                            <span>Storage</span>
                        </button>
                        <button
                                onclick={openAdminMonitor}
                                class="w-full flex items-center gap-3 px-4 py-2 text-xs text-slate-400 hover:text-slate-100 hover:bg-slate-900 transition-colors"
                        >
                            <LineChart size={14}/>
                            <span>Monitor Conversations</span>
                        </button>
                        <button
                                onclick={openUserDirectory}
                                class="w-full flex items-center gap-3 px-4 py-2 text-xs text-slate-400 hover:text-slate-100 hover:bg-slate-900 transition-colors"
                        >
                            <User size={14}/>
                            <span>User Directory</span>
                        </button>
                        <button
                                onclick={openRedisTest}
                                class="w-full flex items-center gap-3 px-4 py-2 text-xs text-slate-400 hover:text-slate-100 hover:bg-slate-900 transition-colors"
                        >
                            <Terminal size={14}/>
                            <span>Redis Pub/Sub Test</span>
                        </button>
                    {/if}
                    <div class="h-px bg-slate-900 my-1"></div>
                    {#if conversationStore.activeConversationId}
                        <button
                                onclick={openConnectionManager}
                                class="w-full flex items-center gap-3 px-4 py-2 text-xs transition-colors {sidebarStore.isPaired ? 'text-blue-400 hover:text-blue-300 hover:bg-blue-900/10' : 'text-slate-400 hover:text-slate-100 hover:bg-slate-900'}"
                        >
                            <Link size={14}/>
                            <span>Linked App</span>
                        </button>

                        <button
                                onclick={openTimeline}
                                class="w-full flex items-center gap-3 px-4 py-2 text-xs text-slate-400 hover:text-slate-100 hover:bg-slate-900 transition-colors"
                        >
                            <Settings2 size={14}/>
                            <span>Soul Timeline</span>
                        </button>

                        {#if profileStore.currentUser?.role === 'admin'}
                            <button
                                    onclick={openStockSignals}
                                    class="w-full flex items-center gap-3 px-4 py-2 text-xs text-slate-400 hover:text-slate-100 hover:bg-slate-900 transition-colors"
                            >
                                <LineChart size={14}/>
                                <span>Stock Signals</span>
                            </button>
                        {/if}

                        <div class="h-px bg-slate-900 my-1"></div>
                    {/if}

                    <button
                            onclick={handleLogout}
                            class="w-full flex items-center gap-3 px-4 py-2 text-xs text-rose-400 hover:text-rose-300 hover:bg-rose-900/20 transition-colors"
                    >
                        <LogOut size={14}/>
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
