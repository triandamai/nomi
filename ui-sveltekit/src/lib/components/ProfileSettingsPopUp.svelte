<script lang="ts">
    import { onMount } from 'svelte';
    import { profileStore } from '$lib/stores/profile.svelte';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { 
        User, Mail, Fingerprint, ShieldCheck, 
        MessageSquare, Link, Save, Loader2, X,
        Smartphone, Hash
    } from 'lucide-svelte';
    import toast from 'svelte-french-toast';

    let displayName = $state('');
    let isUpdating = $state(false);

    onMount(async () => {
        displayName = profileStore.currentUser?.display_name || '';
        await profileStore.fetchUserConnections();
    });

    async function handleUpdate() {
        if (!displayName.trim()) {
            toast.error('Display name cannot be empty');
            return;
        }

        isUpdating = true;
        try {
            await profileStore.updateProfile(displayName.trim());
            toast.success('Profile updated successfully');
            popupStore.closeLast();
        } catch (e: any) {
            toast.error(e.message || 'Failed to update profile');
        } finally {
            isUpdating = false;
        }
    }
</script>

<div class="space-y-8 py-2 text-slate-200">
    <!-- User Profile Header -->
    <div class="flex items-center gap-6">
        <div class="w-20 h-20 rounded-[32px] bg-blue-600/20 border border-blue-500/30 flex items-center justify-center text-3xl font-black text-blue-400 shadow-2xl shadow-blue-500/10 shrink-0">
            {(profileStore.currentUser?.display_name || 'U').charAt(0).toUpperCase()}
        </div>
        <div class="min-w-0 space-y-1">
            <h3 class="text-xl font-black text-white truncate tracking-tight uppercase">
                {profileStore.currentUser?.display_name || 'Anonymous User'}
            </h3>
            <div class="flex items-center gap-2">
                <span class="px-2 py-0.5 rounded bg-blue-500/10 border border-blue-500/20 text-blue-400 text-[9px] font-black uppercase tracking-widest">
                    {profileStore.currentUser?.role || 'User'}
                </span>
                <div class="flex items-center gap-1 text-[10px] text-slate-500 font-mono italic truncate">
                    <Fingerprint size={10} />
                    {profileStore.currentUser?.id}
                </div>
            </div>
        </div>
    </div>

    <!-- Edit Display Name -->
    <div class="bg-slate-950/40 border border-slate-800 rounded-3xl p-6 space-y-4">
        <div class="flex items-center gap-2 px-1">
            <User size={14} class="text-blue-500" />
            <span class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-400">Display Identity</span>
        </div>
        <div class="relative group">
            <input 
                type="text" 
                bind:value={displayName}
                placeholder="How should Nomi address you?"
                class="w-full bg-slate-900 border border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold text-white focus:outline-none focus:ring-2 focus:ring-blue-500/30 transition-all placeholder:text-slate-700"
            />
            <div class="absolute right-4 top-1/2 -translate-y-1/2 opacity-0 group-focus-within:opacity-100 transition-opacity">
                <button 
                    onclick={handleUpdate}
                    disabled={isUpdating || !displayName.trim() || displayName === profileStore.currentUser?.display_name}
                    class="p-2 bg-blue-600 hover:bg-blue-500 disabled:opacity-30 rounded-xl text-white transition-all shadow-lg shadow-blue-500/20"
                >
                    {#if isUpdating} <Loader2 size={14} class="animate-spin" /> {:else} <Save size={14} /> {/if}
                </button>
            </div>
        </div>
        <div class="flex items-center gap-2 px-1 text-[10px] text-slate-600 font-medium">
            <Mail size={12} />
            <span>Identity Source: {profileStore.currentUser?.external_id}</span>
        </div>
    </div>

    <!-- Connections Overview -->
    <div class="grid md:grid-cols-2 gap-6">
        <!-- Channels -->
        <div class="space-y-4">
            <div class="flex items-center gap-2 px-1">
                <Link size={14} class="text-emerald-500" />
                <span class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-400">Linked Channels</span>
            </div>
            <div class="space-y-2">
                {#if profileStore.userChannels.length === 0}
                    <div class="p-6 text-center bg-slate-900/20 border border-dashed border-slate-800 rounded-2xl italic text-xs text-slate-600">
                        No active channels.
                    </div>
                {:else}
                    {#each profileStore.userChannels as channel}
                        <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-2xl flex items-center gap-3 group hover:border-emerald-500/20 transition-all">
                            <div class="w-8 h-8 rounded-lg bg-slate-800 flex items-center justify-center text-slate-400 group-hover:bg-emerald-500/10 group-hover:text-emerald-400 transition-colors">
                                {#if channel.platform === 'whatsapp'}
                                    <svg viewBox="0 0 24 24" class="w-4 h-4 fill-current"><path d="M12.04 2c-5.46 0-9.91 4.45-9.91 9.91 0 1.75.46 3.45 1.32 4.95L2.05 22l5.25-1.38c1.45.79 3.08 1.21 4.74 1.21 5.46 0 9.91-4.45 9.91-9.91 0-2.65-1.03-5.14-2.9-7.01A9.817 9.817 0 0012.04 2m.01 1.67c2.2 0 4.26.86 5.82 2.42 1.56 1.56 2.41 3.63 2.41 5.83 0 4.54-3.7 8.23-8.24 8.23-1.48 0-2.93-.39-4.19-1.15l-.3-.17-3.12.82.83-3.04-.19-.3a8.132 8.132 0 01-1.26-4.38c.01-4.54 3.7-8.24 8.24-8.24m-3.53 4.75c-.19 0-.52.07-.79.37-.27.3-.87.85-.87 2.08s.89 2.42 1.01 2.58c.12.16 1.75 2.67 4.23 3.74.59.26 1.05.41 1.41.52.59.19 1.13.16 1.56.1.48-.07 1.47-.6 1.67-1.18.21-.58.21-1.07.14-1.18-.06-.1-.23-.16-.48-.27-.25-.12-1.47-.73-1.69-.82-.23-.09-.39-.12-.56.12-.17.25-.64.81-.78.97-.14.17-.29.19-.54.06-.25-.12-1.05-.39-1.99-1.23-.74-.66-1.23-1.47-1.38-1.72-.14-.25-.01-.39.11-.51.11-.11.25-.29.37-.43.12-.14.17-.25.25-.41.08-.16.04-.31-.02-.43-.06-.12-.56-1.35-.77-1.85-.2-.5-.4-.43-.56-.44l-.48-.01z"/></svg>
                                {:else if channel.platform === 'telegram'}
                                    <Smartphone size={16} />
                                {:else}
                                    <Hash size={16} />
                                {/if}
                            </div>
                            <div class="min-w-0">
                                <p class="text-[10px] font-black uppercase tracking-widest text-white">{channel.platform}</p>
                                <p class="text-[9px] text-slate-500 font-mono truncate">{channel.paired ? 'CONNECTED' : 'STANDBY'}</p>
                            </div>
                        </div>
                    {/each}
                {/if}
            </div>
        </div>

        <!-- Conversations -->
        <div class="space-y-4">
            <div class="flex items-center gap-2 px-1">
                <MessageSquare size={14} class="text-purple-500" />
                <span class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-400">Conversations</span>
            </div>
            <div class="space-y-2 max-h-[250px] overflow-y-auto pr-2 custom-scrollbar">
                {#if profileStore.userConversations.length === 0}
                    <div class="p-6 text-center bg-slate-900/20 border border-dashed border-slate-800 rounded-2xl italic text-xs text-slate-600">
                        No active conversations.
                    </div>
                {:else}
                    {#each profileStore.userConversations as conv}
                        <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-2xl group hover:border-purple-500/20 transition-all cursor-default">
                            <p class="text-xs font-bold text-white truncate uppercase tracking-tight group-hover:text-purple-300 transition-colors">
                                {conv.name || 'Private Sandbox'}
                            </p>
                            <div class="flex items-center gap-2 mt-2">
                                <div class="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.4)]"></div>
                                <span class="text-[9px] text-slate-500 font-black uppercase tracking-widest">Active Session</span>
                            </div>
                        </div>
                    {/each}
                {/if}
            </div>
        </div>
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: #1e293b; border-radius: 10px; }
</style>
