<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { api, chatApi } from '$lib/api/client';
    import { eventBus } from '$lib/utils';
    import { 
        Play, 
        RotateCcw, 
        Send, 
        Brain, 
        Sparkles, 
        User, 
        Smartphone, 
        CheckCircle2, 
        AlertCircle, 
        Clock, 
        ArrowRightLeft,
        Lock,
        ArrowRight,
        Activity,
        Cpu
    } from 'lucide-svelte';

    // Simulation Configs
    const PARENT_CONVO_ID = '99999999-8888-7777-6666-555555555555';
    const SUB_CONVO_ID = '88888888-7777-6666-5555-444444444444';
    const TASK_ID = '44444444-3333-2222-1111-000000000000';

    let isSeeding = $state(false);
    let isSending = $state(false);
    let isCancelling = $state(false);
    let errorMsg = $state('');
    
    // Live database values fetched via polling & MQTT
    let htoState = $state<'idle' | 'running' | 'waiting_external_feedback' | 'paused_for_input' | 'completed' | 'failed'>('idle');
    let taskTitle = $state('Seeding simulation...');
    let globalGoal = $state('');
    let currentStepIndex = $state(0);
    let checkpoints = $state<any[]>([]);

    let ownerMessages = $state<{role: 'user'|'assistant'|'system', content: string, created_at: string}[]>([]);
    let targetMessages = $state<{role: 'user'|'assistant'|'system', content: string, created_at: string}[]>([]);
    let logs = $state<{step_index: number, event_type: string, log_content: string, created_at: string}[]>([]);

    // Form inputs
    let ownerInput = $state('');
    let targetInput = $state('');
    let pollingInterval: any = null;
    let unsubscribes: (() => void)[] = [];

    // Seeding API to reset database state
    async function triggerDbSeed() {
        isSeeding = true;
        errorMsg = '';
        try {
            await api.post('/v1/admin/tasks/simulation/seed');
            await fetchLiveData();
            await fetchInitialMessages();
        } catch (e: any) {
            errorMsg = e.message || 'Seeding failed. Ensure backend server is running and database is fully migrated.';
        } finally {
            isSeeding = false;
        }
    }

    // Force cancel the autonomous task gracefully
    async function forceCancelTask() {
        if (isCancelling) return;
        isCancelling = true;
        errorMsg = '';
        try {
            await api.post(`/tasks/${TASK_ID}/cancel`);
            await fetchLiveData();
        } catch (e: any) {
            errorMsg = e.message || 'Failed to force cancel the task.';
        } finally {
            isCancelling = false;
        }
    }

    // Polling function to pull live task logs and states ONLY (no messages, resolved polling spam!)
    async function fetchLiveData() {
        try {
            // 1. Fetch Timeline details (checkpoints, status, logs)
            const taskRes: any = await api.get(`/tasks/${TASK_ID}/timeline`);
            if (taskRes && taskRes.data) {
                taskTitle = taskRes.data.title;
                globalGoal = taskRes.data.global_goal;
                htoState = taskRes.data.status;
                currentStepIndex = taskRes.data.current_step_index;
                checkpoints = taskRes.data.checkpoints || [];
                logs = taskRes.data.logs || [];
            }
        } catch (e) {
            console.error("Error polling simulation live data:", e);
        }
    }

    // Fetch conversation message history ONCE (bypassing membership check via our custom admin simulation API!)
    async function fetchInitialMessages() {
        try {
            // Fetch Owner private room messages
            const ownerRes: any = await api.get(`/v1/admin/tasks/simulation/messages/${PARENT_CONVO_ID}`);
            if (ownerRes && ownerRes.data?.messages) {
                ownerMessages = [...ownerRes.data.messages].reverse();
            }

            // Fetch WhatsApp target sub-chat messages
            const targetRes: any = await api.get(`/v1/admin/tasks/simulation/messages/${SUB_CONVO_ID}`);
            if (targetRes && targetRes.data?.messages) {
                targetMessages = [...targetRes.data.messages].reverse();
            }
        } catch (e) {
            console.error("Error fetching simulation message histories:", e);
        }
    }

    // Send mock inbound WhatsApp channel message
    async function submitTargetReply(e: Event) {
        e.preventDefault();
        if (!targetInput.trim() || isSending) return;

        const text = targetInput;
        targetInput = '';
        isSending = true;

        try {
            await chatApi.publishAdminInbound({
                event: "nomi:inbound",
                channel: "whatsapp",
                payload: {
                    is_group: false,
                    is_private: true,
                    is_mentioned: true,
                    sender_id: "triandamai@s.whatsapp.net",
                    conversation_id: "triandamai@s.whatsapp.net",
                    message_id: crypto.randomUUID(),
                    text: text,
                    channel: "whatsapp"
                }
            });
            await fetchLiveData();
        } catch (e: any) {
            errorMsg = e.message || 'Failed to submit mock WhatsApp inbound message.';
        } finally {
            isSending = false;
        }
    }

    // Send mock inbound Owner Telegram/Web channel message
    async function submitOwnerReply(e: Event) {
        e.preventDefault();
        if (!ownerInput.trim() || isSending) return;

        const text = ownerInput;
        ownerInput = '';
        isSending = true;

        try {
            await chatApi.publishAdminInbound({
                event: "nomi:inbound",
                channel: "telegram",
                payload: {
                    is_group: false,
                    is_private: true,
                    is_mentioned: true,
                    sender_id: "telegram_owner_123",
                    conversation_id: "telegram_owner_123",
                    message_id: crypto.randomUUID(),
                    text: text,
                    channel: "telegram"
                }
            });
            await fetchLiveData();
        } catch (e: any) {
            errorMsg = e.message || 'Failed to submit mock Owner inbound message.';
        } finally {
            isSending = false;
        }
    }

    onMount(async () => {
        // Run seed first to set up the DB state automatically!
        await triggerDbSeed();
        await fetchInitialMessages();
        
        // Poll every 3 seconds ONLY for state checklist and task logs
        pollingInterval = setInterval(fetchLiveData, 3000);

        // Real-time MQTT pushes! Whenever a new message, tool or thought is streamed, instantly append it!
        unsubscribes.push(
            eventBus.subscribe('sse-message', (data: any) => {
                if (data && data.conversation_id === PARENT_CONVO_ID) {
                    ownerMessages = [...ownerMessages, data];
                    fetchLiveData();
                } else if (data && data.conversation_id === SUB_CONVO_ID) {
                    targetMessages = [...targetMessages, data];
                    fetchLiveData();
                }
            })
        );
        unsubscribes.push(
            eventBus.subscribe('sse-thought', (data: any) => {
                if (data && (data.conversation_id === PARENT_CONVO_ID || data.conversation_id === SUB_CONVO_ID)) {
                    fetchLiveData();
                }
            })
        );
        unsubscribes.push(
            eventBus.subscribe('sse-tool_start', (data: any) => {
                fetchLiveData();
            })
        );
        unsubscribes.push(
            eventBus.subscribe('sse-tool_end', (data: any) => {
                fetchLiveData();
            })
        );
    });

    onDestroy(() => {
        if (pollingInterval) clearInterval(pollingInterval);
        unsubscribes.forEach(unsub => unsub());
    });
</script>

<div class="flex-1 flex flex-col bg-slate-950 p-6 overflow-y-auto min-h-0 select-none">
    <!-- Header -->
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6 border-b border-slate-900 pb-5">
        <div>
            <div class="flex items-center gap-2 mb-1">
                <div class="p-1.5 bg-blue-500/10 rounded-lg text-blue-400 border border-blue-500/20">
                    <Activity size={18} class="animate-pulse" />
                </div>
                <h1 class="text-xs uppercase font-black tracking-[0.2em] text-slate-400">Live Workspace HTO Control Panel</h1>
            </div>
            <h2 class="text-xl font-black text-slate-100">Hierarchical Task Orchestrator & Live Thread Resurrection Engine</h2>
        </div>
        <div class="flex items-center gap-3">
            <button 
                onclick={triggerDbSeed}
                disabled={isSeeding}
                class="flex items-center gap-2 px-5 py-2.5 bg-blue-600 hover:bg-blue-500 disabled:opacity-40 text-xs font-black uppercase tracking-widest rounded-xl text-white transition-all shadow-lg shadow-blue-500/20 active:scale-95"
            >
                <RotateCcw size={14} class={isSeeding ? 'animate-spin' : ''} />
                <span>Reset Simulation DB State</span>
            </button>
        </div>
    </div>

    {#if errorMsg}
        <div class="mb-5 p-4 rounded-2xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs flex items-center gap-3 animate-in fade-in slide-in-from-top-2 duration-200">
            <AlertCircle size={16} />
            <span>{errorMsg}</span>
        </div>
    {/if}

    <!-- Simulation Details Card -->
    <div class="bg-slate-900/20 border border-slate-900 rounded-3xl p-5 mb-6 flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
            <div class="flex items-center gap-2.5 mb-1.5">
                <span class="text-xs font-black uppercase tracking-widest text-blue-400">Active Task Details</span>
                <span class="text-[9px] font-mono bg-slate-950 text-slate-500 px-2 py-0.5 rounded border border-slate-800">UUID: {TASK_ID}</span>
            </div>
            <h3 class="text-md font-bold text-slate-100">{taskTitle}</h3>
            <p class="text-xs text-slate-400 mt-1 max-w-3xl leading-relaxed">
                <strong class="text-slate-300">Global Goal:</strong> {globalGoal || 'Seeding autonomous task environment...'}
            </p>
        </div>
        <div class="flex flex-col gap-2 w-full md:w-auto">
            <div class="flex items-center justify-between gap-4">
                <span class="text-[10px] uppercase font-black tracking-widest text-slate-500">Task Checkpoints:</span>
                <span class="text-[10px] font-mono text-slate-400">{checkpoints.length} defined</span>
            </div>
            <div class="flex gap-2">
                {#each checkpoints as cp, idx}
                    <div class="flex items-center gap-1.5 px-3 py-1 bg-slate-950/60 border border-slate-900 rounded-lg">
                        <div class="w-1.5 h-1.5 rounded-full {cp.status === 'completed' ? 'bg-emerald-400' : 'bg-amber-400'}"></div>
                        <span class="text-[10px] text-slate-300 font-bold">{cp.step || `Step ${idx+1}`}</span>
                    </div>
                {/each}
            </div>
        </div>
    </div>

    <!-- Main Workspace Panoramic Layout -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 flex-1 min-h-0">
        
        <!-- Column 1: Owner Chat (Parent Room - Trian Room) -->
        <div class="lg:col-span-4 flex flex-col bg-slate-900/40 border border-slate-900 rounded-3xl overflow-hidden min-h-[480px]">
            <div class="px-4 py-3 bg-slate-900/80 border-b border-slate-900 flex items-center justify-between">
                <div class="flex items-center gap-2.5">
                    <div class="w-8 h-8 rounded-full bg-blue-500/10 flex items-center justify-center text-blue-400 font-bold text-xs border border-blue-500/20">
                        <User size={14} />
                    </div>
                    <div>
                        <h3 class="text-xs font-black uppercase tracking-widest text-slate-200">Owner (Trian)</h3>
                        <p class="text-[9px] text-slate-500">Parent Conversation ID: {PARENT_CONVO_ID.substring(0,8)}...</p>
                    </div>
                </div>
                {#if htoState === 'paused_for_input'}
                    <span class="flex items-center gap-1 text-[9px] font-black uppercase tracking-widest px-2 py-0.5 rounded-md bg-amber-500/10 text-amber-400 border border-amber-500/20 animate-pulse">
                        <AlertCircle size={10} />
                        Needs Action
                    </span>
                {/if}
            </div>

            <!-- Messages Area -->
            <div class="flex-1 p-4 overflow-y-auto space-y-4 min-h-0 flex flex-col justify-end">
                {#if ownerMessages.length === 0}
                    <div class="text-center py-12 text-slate-700">
                        <Smartphone size={32} class="mx-auto mb-2 opacity-30" />
                        <p class="text-xs">Messages in Trian's main room display here.</p>
                    </div>
                {:else}
                    {#each ownerMessages as msg}
                        <div class="flex flex-col {msg.role === 'user' ? 'items-end' : 'items-start'}">
                            <div class="max-w-[85%] rounded-2xl px-4 py-2.5 text-xs leading-relaxed {msg.role === 'user' ? 'bg-blue-600 text-white rounded-tr-none' : 'bg-slate-950 text-slate-200 rounded-tl-none border border-slate-800'}">
                                {msg.content}
                            </div>
                            <span class="text-[8px] text-slate-600 mt-1">{new Date(msg.created_at || Date.now()).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}</span>
                        </div>
                    {/each}
                {/if}
            </div>

            <!-- Composer input -->
            <form onsubmit={submitOwnerReply} class="p-4 border-t border-slate-900 bg-slate-950/40">
                <div class="flex gap-2">
                    <input 
                        type="text" 
                        bind:value={ownerInput}
                        disabled={isSending || htoState !== 'paused_for_input'}
                        placeholder={htoState === 'paused_for_input' ? 'Respond as Trian the Owner...' : 'Waiting for Nomi to request input...'}
                        class="flex-1 bg-slate-950 border border-slate-900 rounded-xl px-4 py-2.5 text-xs text-slate-200 focus:outline-none focus:border-blue-500/50 disabled:opacity-40 placeholder:text-slate-700"
                    />
                    <button 
                        type="submit" 
                        disabled={!ownerInput.trim() || isSending || htoState !== 'paused_for_input'}
                        class="px-4 bg-blue-600 text-white rounded-xl hover:bg-blue-500 disabled:opacity-30 transition-all flex items-center justify-center"
                    >
                        <Send size={12} />
                    </button>
                </div>
            </form>
        </div>

        <!-- Column 2: HTO Brain & Real-Time Engine Logs (Center) -->
        <div class="lg:col-span-4 flex flex-col gap-4">
            <!-- HTO State Panel -->
            <div class="bg-slate-900/40 border border-slate-900 rounded-3xl p-5 flex flex-col justify-center relative overflow-hidden">
                <div class="flex items-center justify-between mb-4">
                    <span class="text-[10px] font-black uppercase tracking-widest text-slate-500">HTO State Machine</span>
                    {#if htoState !== 'idle' && htoState !== 'completed' && htoState !== 'failed'}
                        <button 
                            onclick={forceCancelTask}
                            disabled={isCancelling}
                            class="text-[9px] font-black uppercase tracking-widest text-red-500 hover:text-red-400 bg-red-500/10 border border-red-500/20 px-2 py-0.5 rounded-lg transition-all active:scale-[0.97] disabled:opacity-50 font-bold"
                        >
                            {#if isCancelling}
                                Cancelling...
                            {:else}
                                Force Cancel
                            {/if}
                        </button>
                    {:else}
                        <span class="text-[9px] font-mono text-slate-700 font-bold">Tokio Engine</span>
                    {/if}
                </div>

                <div class="flex items-center gap-4 py-2">
                    <!-- Dynamic state circle animation -->
                    <div class="relative w-12 h-12 shrink-0 rounded-full flex items-center justify-center border-2 border-dashed {htoState === 'running' ? 'border-blue-500 animate-spin' : htoState === 'waiting_external_feedback' ? 'border-purple-500 animate-pulse' : htoState === 'paused_for_input' ? 'border-amber-500 animate-pulse' : htoState === 'completed' ? 'border-emerald-500' : 'border-slate-800'}">
                        <div class="w-8 h-8 rounded-full flex items-center justify-center {htoState === 'running' ? 'bg-blue-500/10 text-blue-400' : htoState === 'waiting_external_feedback' ? 'bg-purple-500/10 text-purple-400' : htoState === 'paused_for_input' ? 'bg-amber-500/10 text-amber-400' : htoState === 'completed' ? 'bg-emerald-500/10 text-emerald-400' : 'bg-slate-900 text-slate-600'}">
                            <Brain size={16} />
                        </div>
                    </div>

                    <div class="min-w-0">
                        <div class="flex items-center gap-2 mb-0.5">
                            <span class="text-sm font-bold uppercase tracking-wider text-slate-200">
                                {htoState === 'idle' ? 'Ready to Seed' : htoState === 'running' ? 'Active Planning' : htoState === 'waiting_external_feedback' ? 'Waiting Target' : htoState === 'paused_for_input' ? 'Waiting Owner' : htoState === 'completed' ? 'Task Completed' : 'Task Failed'}
                            </span>
                        </div>
                        <p class="text-[10px] text-slate-500 leading-normal">
                            {htoState === 'idle' ? 'Seeding simulation workspace database...' : htoState === 'running' ? 'Nomi executing background API planning loop...' : htoState === 'waiting_external_feedback' ? 'HTO sleeping. Target needs to reply on WA JID.' : htoState === 'paused_for_input' ? 'HTO sleeping. Owner needs to reply in Parent Room.' : htoState === 'completed' ? 'Goal successfully satisfied! 🏁' : 'Task failed or canceled.'}
                        </p>
                    </div>
                </div>
            </div>

            <!-- Real-Time Database Timeline Logs -->
            <div class="flex-1 flex flex-col bg-slate-900/40 border border-slate-900 rounded-3xl p-5 overflow-hidden">
                <div class="flex items-center justify-between mb-4 border-b border-slate-900 pb-3">
                    <span class="text-[10px] font-black uppercase tracking-widest text-slate-500">Live database audit timeline</span>
                    <span class="text-[9px] font-mono text-slate-700 font-bold">autonomous_task_logs</span>
                </div>

                <div class="flex-1 overflow-y-auto space-y-3 pr-1">
                    {#if logs.length === 0}
                        <div class="text-center py-16 text-slate-700">
                            <Clock size={28} class="mx-auto mb-2 opacity-30" />
                            <p class="text-xs">Database audit timeline events will display chronologically.</p>
                        </div>
                    {:else}
                        {#each logs as log}
                            <div class="flex gap-3 bg-slate-950/40 border border-slate-900 p-3 rounded-2xl transition-all hover:bg-slate-950">
                                <div class="shrink-0 mt-0.5">
                                    {#if log.event_type === 'step_start'}
                                        <div class="p-1 rounded bg-blue-500/10 text-blue-400 border border-blue-500/10">
                                            <Play size={10} />
                                        </div>
                                    {:else if log.event_type === 'tool_execution'}
                                        <div class="p-1 rounded bg-purple-500/10 text-purple-400 border border-purple-500/10">
                                            <Sparkles size={10} />
                                        </div>
                                    {:else if log.event_type === 'outbound_msg'}
                                        <div class="p-1 rounded bg-amber-500/10 text-amber-400 border border-amber-500/10">
                                            <ArrowRight size={10} />
                                        </div>
                                    {:else if log.event_type === 'external_feedback' || log.event_type === 'human_response'}
                                        <div class="p-1 rounded bg-violet-500/10 text-violet-400 border border-violet-500/10">
                                            <ArrowRightLeft size={10} />
                                        </div>
                                    {:else if log.event_type === 'completed' || log.event_type === 'step_end'}
                                        <div class="p-1 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/10">
                                            <CheckCircle2 size={10} />
                                        </div>
                                    {/if}
                                </div>
                                <div class="min-w-0">
                                    <div class="flex items-center gap-2 mb-1">
                                        <span class="text-[8px] font-black uppercase tracking-widest text-slate-500">{log.event_type}</span>
                                        <span class="text-[8px] font-mono text-slate-700">Step {log.step_index}</span>
                                    </div>
                                    <p class="text-[11px] text-slate-300 leading-relaxed font-sans">{log.log_content}</p>
                                    <span class="text-[8px] text-slate-600 block mt-1">{new Date(log.created_at || Date.now()).toLocaleTimeString()}</span>
                                </div>
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>
        </div>

        <!-- Column 3: Target Chat (WhatsApp Mock - Triandamai Room) -->
        <div class="lg:col-span-4 flex flex-col bg-slate-900/40 border border-slate-900 rounded-3xl overflow-hidden min-h-[480px]">
            <div class="px-4 py-3 bg-[#075e54]/20 border-b border-[#075e54]/30 flex items-center justify-between">
                <div class="flex items-center gap-2.5">
                    <div class="w-8 h-8 rounded-full bg-[#128c7e]/10 flex items-center justify-center text-[#128c7e] font-bold text-xs border border-[#128c7e]/20">
                        <Smartphone size={14} />
                    </div>
                    <div>
                        <h3 class="text-xs font-black uppercase tracking-widest text-slate-200">Target (Triandamai)</h3>
                        <p class="text-[9px] text-[#128c7e] font-bold">WhatsApp Channel ID: {SUB_CONVO_ID.substring(0,8)}...</p>
                    </div>
                </div>
                {#if htoState === 'waiting_external_feedback'}
                    <span class="flex items-center gap-1 text-[9px] font-black uppercase tracking-widest px-2 py-0.5 rounded-md bg-[#25d366]/10 text-[#25d366] border border-[#25d366]/20 animate-pulse">
                        <Clock size={10} />
                        Waiting Target
                    </span>
                {/if}
            </div>

            <!-- Messages Area -->
            <div class="flex-1 p-4 overflow-y-auto space-y-4 min-h-0 flex flex-col justify-end">
                {#if targetMessages.length === 0}
                    <div class="text-center py-12 text-slate-700">
                        <Smartphone size={32} class="mx-auto mb-2 opacity-30 text-emerald-800" />
                        <p class="text-xs">Replies from target JID on WhatsApp display here.</p>
                    </div>
                {:else}
                    {#each targetMessages as msg}
                        <div class="flex flex-col {msg.role === 'user' ? 'items-end' : 'items-start'}">
                            <div class="max-w-[85%] rounded-2xl px-4 py-2.5 text-xs leading-relaxed {msg.role === 'user' ? 'bg-[#056162] text-white rounded-tr-none' : 'bg-slate-950 text-slate-200 rounded-tl-none border border-slate-800'}">
                                {msg.content}
                            </div>
                            <span class="text-[8px] text-slate-600 mt-1">{new Date(msg.created_at || Date.now()).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}</span>
                        </div>
                    {/each}
                {/if}
            </div>

            <!-- Composer input -->
            <form onsubmit={submitTargetReply} class="p-4 border-t border-slate-900 bg-slate-950/40">
                <div class="flex gap-2">
                    <input 
                        type="text" 
                        bind:value={targetInput}
                        disabled={isSending || htoState !== 'waiting_external_feedback'}
                        placeholder={htoState === 'waiting_external_feedback' ? 'Reply as Target Triandamai...' : 'Waiting for Nomi to message target...'}
                        class="flex-1 bg-slate-950 border border-slate-900 rounded-xl px-4 py-2.5 text-xs text-slate-200 focus:outline-none focus:border-[#128c7e]/50 disabled:opacity-40 placeholder:text-slate-700"
                    />
                    <button 
                        type="submit" 
                        disabled={!targetInput.trim() || isSending || htoState !== 'waiting_external_feedback'}
                        class="px-4 bg-[#128c7e] text-white rounded-xl hover:bg-[#075e54] disabled:opacity-30 transition-all flex items-center justify-center"
                    >
                        <Send size={12} />
                    </button>
                </div>
            </form>
        </div>

    </div>
</div>
