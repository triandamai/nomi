<script lang="ts">
    import { Plus, MoreVertical, Edit2, Trash2 } from 'lucide-svelte';
    import Avatar from './Avatar.svelte';
    import { conversationStore, type Conversation } from '$lib/stores/conversation.svelte';
    import { profileStore } from '$lib/stores/profile.svelte';
    import { popupStore } from '$lib/stores/popup.svelte';

    let newConvName = $state('');
    let editingConv = $state<Conversation | null>(null);

    function handleAddConversation() {
        newConvName = '';
        popupStore.open({
            title: 'Create Conversation',
            width: 'max-w-md',
            contentSnippet: createConvContent,
            footerSnippet: createConvFooter
        });
    }

    function handleEditConversation(conv: Conversation) {
        editingConv = conv;
        newConvName = conv.name;
        popupStore.open({
            title: 'Edit Conversation',
            width: 'max-w-md',
            contentSnippet: editConvContent,
            footerSnippet: editConvFooter
        });
    }

    function handleDeleteConversation(conv: Conversation) {
        editingConv = conv;
        popupStore.open({
            title: 'Delete Conversation',
            width: 'max-w-sm',
            contentSnippet: deleteConvContent,
            footerSnippet: deleteConvFooter
        });
    }

    async function createConversation() {
        if (newConvName.trim()) {
            await conversationStore.addConversation(newConvName.trim());
            newConvName = '';
            popupStore.closeLast();
        }
    }

    async function updateConversation() {
        if (editingConv && newConvName.trim()) {
            await conversationStore.updateConversation(editingConv.id, newConvName.trim());
            editingConv = null;
            newConvName = '';
            popupStore.closeLast();
        }
    }

    async function deleteConversation() {
        if (editingConv) {
            await conversationStore.deleteConversation(editingConv.id);
            editingConv = null;
            popupStore.closeLast();
        }
    }
</script>

{#snippet createConvContent()}
    <div class="space-y-4">
        <p class="text-xs text-zinc-500 uppercase font-bold tracking-widest">Conversation Name</p>
        <input
            type="text"
            bind:value={newConvName}
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
            disabled={!newConvName.trim()}
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
            bind:value={newConvName}
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
            disabled={!newConvName.trim()}
            class="px-6 py-2 text-xs font-bold uppercase tracking-wider bg-emerald-600 hover:bg-emerald-500 rounded-lg text-white transition-all"
        >
            Save Changes
        </button>
    </div>
{/snippet}

{#snippet deleteConvContent()}
    <div class="space-y-4">
        <p class="text-sm text-zinc-300">
            Are you sure you want to delete <span class="font-bold text-zinc-100">{editingConv?.name}</span>? This action cannot be undone.
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
        <button
            onclick={handleAddConversation}
            class="w-12 h-12 rounded-[24px] hover:rounded-[16px] bg-zinc-800 hover:bg-emerald-600 text-emerald-500 hover:text-white flex items-center justify-center transition-all group relative"
        >
            <Plus class="w-6 h-6" />
            <div class="absolute left-16 px-3 py-1 bg-zinc-950 text-white text-xs font-bold rounded shadow-xl whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity z-50">
                Add Conversation
            </div>
        </button>

        <!-- Current User -->
        <Avatar
            name={profileStore.currentUser.name}
            active={false}
            online={profileStore.currentUser.status === 'online'}
            onClick={() => {}}
        />
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
