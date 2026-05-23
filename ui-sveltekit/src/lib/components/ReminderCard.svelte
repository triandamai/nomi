<script lang="ts">
    import { onMount } from 'svelte';
    import { Bell, Clock, Calendar, CheckCircle2, AlertCircle } from 'lucide-svelte';
    import { api } from '$lib/api/client';

    let { ref_id } = $props();

    let reminder = $state<any>(null);
    let loading = $state(true);
    let error = $state<string | null>(null);

    onMount(async () => {
        try {
            const res = await api.get<any>(`/reminders/${ref_id}`);
            if (res.data) {
                reminder = res.data;
            } else {
                error = res.meta?.message || "Reminder not found";
            }
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    });

    function formatTime(dateStr: string) {
        return new Date(dateStr).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }

    function formatDate(dateStr: string) {
        return new Date(dateStr).toLocaleDateString([], { month: 'short', day: 'numeric' });
    }
</script>

{#if loading}
    <div class="p-4 bg-slate-900/40 border border-slate-800 rounded-2xl animate-pulse flex items-center gap-3">
        <div class="w-10 h-10 bg-slate-800 rounded-full"></div>
        <div class="flex-1 space-y-2">
            <div class="h-2 bg-slate-800 rounded w-1/2"></div>
            <div class="h-2 bg-slate-800 rounded w-1/4"></div>
        </div>
    </div>
{:else if error}
    <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-2xl flex items-center gap-3 text-red-400 text-xs italic">
        <AlertCircle class="w-4 h-4" />
        <span>Failed to load reminder details.</span>
    </div>
{:else if reminder}
    <div class="bg-slate-900/60 border border-emerald-500/30 rounded-2xl overflow-hidden shadow-xl backdrop-blur-md group hover:border-emerald-500/50 transition-all duration-300 max-w-sm">
        <div class="px-4 py-2 border-b border-white/5 bg-emerald-500/10 flex items-center justify-between">
            <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-emerald-400">
                <Bell class="w-3 h-3" />
                Active Reminder
            </div>
            <div class="px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-400 text-[8px] font-bold uppercase">
                {reminder.status}
            </div>
        </div>
        
        <div class="p-4 flex flex-col gap-3">
            <p class="text-sm font-medium text-white line-clamp-2 leading-relaxed">
                {reminder.content}
            </p>
            
            <div class="flex items-center gap-4 text-[10px] text-slate-400 font-bold uppercase tracking-wider">
                <div class="flex items-center gap-1.5">
                    <Clock class="w-3 h-3 text-emerald-400" />
                    {formatTime(reminder.due_at)}
                </div>
                <div class="flex items-center gap-1.5">
                    <Calendar class="w-3 h-3 text-emerald-400" />
                    {formatDate(reminder.due_at)}
                </div>
            </div>

            {#if reminder.frequency !== 'once'}
                <div class="flex items-center gap-1.5 text-[9px] text-emerald-400/60 font-medium">
                    <CheckCircle2 class="w-3 h-3" />
                    Repeats: {reminder.frequency}
                </div>
            {/if}
        </div>
    </div>
{/if}
