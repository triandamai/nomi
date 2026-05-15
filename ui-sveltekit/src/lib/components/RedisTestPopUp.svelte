<script lang="ts">
    import { chatApi } from '$lib/api/client';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { Send, Loader2, Database, Terminal, MessageSquare, AlertCircle } from 'lucide-svelte';

    let activeTab = $state<'inbound' | 'outbound'>('inbound');
    let isLoading = $state(false);
    let result = $state<{ success: boolean; message: string } | null>(null);

    // Inbound data matches RedisInboundRequest
    let inboundData = $state({
        event: 'nomi:inbound',
        channel: 'telegram',
        payload: {
            is_group: false,
            is_private: true,
            is_mentioned: false,
            sender_id: '',
            conversation_id: '',
            message_id: 'test-' + Date.now(),
            text: 'Test inbound message',
            channel: 'telegram',
            metadata: {}
        }
    });

    // Outbound data matches RedisOutboundRequest
    let outboundData = $state({
        channel: 'telegram',
        event: 'nomi:outbound',
        conversation_id: '',
        role: 'assistant',
        content: 'Test outbound message'
    });

    async function handlePublish() {
        isLoading = true;
        result = null;
        try {
            if (activeTab === 'inbound') {
                const res = await chatApi.publishAdminInbound(inboundData);
                result = { success: true, message: res.meta?.message || 'Inbound event published' };
            } else {
                const res = await chatApi.publishAdminOutbound(outboundData);
                result = { success: true, message: res.meta?.message || 'Outbound event published' };
            }
        } catch (e: any) {
            console.error(e);
            result = { success: false, message: e.message || 'Failed to publish event' };
        } finally {
            isLoading = false;
        }
    }
</script>

<div class="flex flex-col h-full text-slate-200">
    <!-- Tabs -->
    <div class="flex p-1 bg-slate-950/50 rounded-xl border border-slate-800 mb-6">
        <button 
            onclick={() => activeTab = 'inbound'}
            class="flex-1 flex items-center justify-center gap-2 py-2 text-xs font-black uppercase tracking-widest rounded-lg transition-all {activeTab === 'inbound' ? 'bg-blue-600 text-white shadow-lg' : 'text-slate-500 hover:text-slate-300'}"
        >
            <MessageSquare size={14} />
            Inbound
        </button>
        <button 
            onclick={() => activeTab = 'outbound'}
            class="flex-1 flex items-center justify-center gap-2 py-2 text-xs font-black uppercase tracking-widest rounded-lg transition-all {activeTab === 'outbound' ? 'bg-purple-600 text-white shadow-lg' : 'text-slate-500 hover:text-slate-300'}"
        >
            <Send size={14} />
            Outbound
        </button>
    </div>

    <div class="flex-1 overflow-y-auto space-y-4 custom-scrollbar pr-1">
        <div class="bg-slate-900/40 border border-slate-800/50 rounded-2xl p-5 space-y-4">
            <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">
                <Terminal size={12} />
                <span>Message Payload</span>
            </div>

            <div class="grid gap-4">
                <div class="space-y-1.5">
                    <label class="text-[9px] font-black uppercase tracking-widest text-slate-600">Conversation ID (UUID)</label>
                    {#if activeTab === 'inbound'}
                        <input 
                            type="text" 
                            bind:value={inboundData.payload.conversation_id}
                            placeholder="00000000-0000-0000-0000-000000000000"
                            class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-xs font-mono text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500/50 transition-all"
                        />
                    {:else}
                        <input 
                            type="text" 
                            bind:value={outboundData.conversation_id}
                            placeholder="00000000-0000-0000-0000-000000000000"
                            class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-xs font-mono text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500/50 transition-all"
                        />
                    {/if}
                </div>

                {#if activeTab === 'inbound'}
                    <div class="space-y-1.5">
                        <label class="text-[9px] font-black uppercase tracking-widest text-slate-600">Sender External ID</label>
                        <input 
                            type="text" 
                            bind:value={inboundData.payload.sender_id}
                            placeholder="e.g. 123456789"
                            class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-xs font-mono text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500/50 transition-all"
                        />
                    </div>
                {:else}
                     <div class="space-y-1.5">
                        <label class="text-[9px] font-black uppercase tracking-widest text-slate-600">Role</label>
                        <select 
                            bind:value={outboundData.role}
                            class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-xs text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500/50 transition-all appearance-none"
                        >
                            <option value="user">User</option>
                            <option value="assistant">Assistant</option>
                            <option value="system">System</option>
                        </select>
                    </div>
                {/if}

                <div class="space-y-1.5">
                    <label class="text-[9px] font-black uppercase tracking-widest text-slate-600">Message Content / Text</label>
                    {#if activeTab === 'inbound'}
                        <textarea 
                            bind:value={inboundData.payload.text}
                            rows="3"
                            class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-xs text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500/50 transition-all resize-none"
                        ></textarea>
                    {:else}
                        <textarea 
                            bind:value={outboundData.content}
                            rows="3"
                            class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-xs text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500/50 transition-all resize-none"
                        ></textarea>
                    {/if}
                </div>

                <div class="grid grid-cols-2 gap-4">
                    <div class="space-y-1.5">
                        <label class="text-[9px] font-black uppercase tracking-widest text-slate-600">Channel</label>
                        {#if activeTab === 'inbound'}
                            <select 
                                bind:value={inboundData.channel}
                                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-xs text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500/50 transition-all appearance-none"
                            >
                                <option value="telegram">Telegram</option>
                                <option value="whatsapp">WhatsApp</option>
                                <option value="web">Web</option>
                            </select>
                        {:else}
                            <select 
                                bind:value={outboundData.channel}
                                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-4 py-2.5 text-xs text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500/50 transition-all appearance-none"
                            >
                                <option value="telegram">Telegram</option>
                                <option value="whatsapp">WhatsApp</option>
                                <option value="web">Web</option>
                            </select>
                        {/if}
                    </div>
                    
                    {#if activeTab === 'inbound'}
                        <div class="flex items-center gap-4 pt-5">
                            <label class="flex items-center gap-2 cursor-pointer group">
                                <input type="checkbox" bind:checked={inboundData.payload.is_mentioned} class="hidden" />
                                <div class="w-4 h-4 rounded border border-slate-700 flex items-center justify-center transition-all {inboundData.payload.is_mentioned ? 'bg-blue-600 border-blue-500' : 'bg-slate-950 group-hover:border-slate-500'}">
                                    {#if inboundData.payload.is_mentioned}
                                        <div class="w-1.5 h-1.5 bg-white rounded-full"></div>
                                    {/if}
                                </div>
                                <span class="text-[10px] font-bold text-slate-500 group-hover:text-slate-300">Mentioned</span>
                            </label>
                        </div>
                    {/if}
                </div>
            </div>
        </div>

        {#if result}
            <div class="p-4 rounded-2xl border flex items-start gap-3 animate-in fade-in slide-in-from-top-2 {result.success ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400' : 'bg-rose-500/10 border-rose-500/20 text-rose-400'}">
                {#if result.success}
                    <Database size={16} class="shrink-0 mt-0.5" />
                {:else}
                    <AlertCircle size={16} class="shrink-0 mt-0.5" />
                {/if}
                <div class="text-xs font-medium">{result.message}</div>
            </div>
        {/if}

        <button 
            onclick={handlePublish} 
            disabled={isLoading || (!inboundData.payload.conversation_id && activeTab === 'inbound') || (!outboundData.conversation_id && activeTab === 'outbound')}
            class="w-full flex items-center justify-center gap-2 py-4 rounded-2xl font-black uppercase tracking-[0.2em] text-xs transition-all active:scale-[0.98] disabled:opacity-50 {activeTab === 'inbound' ? 'bg-blue-600 hover:bg-blue-500 shadow-lg shadow-blue-500/20' : 'bg-purple-600 hover:bg-purple-500 shadow-lg shadow-purple-500/20'} text-white"
        >
            {#if isLoading}
                <Loader2 size={16} class="animate-spin" />
                Publishing...
            {:else}
                <Send size={16} />
                Publish to Redis
            {/if}
        </button>
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: #1e293b; border-radius: 10px; }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: #334155; }
</style>
