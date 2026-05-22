<script lang="ts">
    import { Brackets, Info, Map, Terminal, Zap } from 'lucide-svelte';
    
    let { data } = $props<{
        data: {
            slug: string;
            name: string;
            description: string;
            schema_json: any;
            how_it_works: string;
            intents: string[];
        }
    }>();
</script>

<div class="flex flex-col gap-6 p-2 text-neutral-200">
    <!-- Header Section -->
    <div class="flex flex-col gap-1">
        <h2 class="text-xl font-bold text-white flex items-center gap-2">
            <Zap class="w-5 h-5 text-accent-emerald" />
            {data.name}
        </h2>
        <p class="text-xs font-mono text-text-muted">{data.slug}</p>
    </div>

    <!-- Intelligence Intent Mapping -->
    <div class="space-y-2">
        <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-[0.2em] text-primary-blue/60">
            <Zap class="w-3.5 h-3.5" />
            Intelligence Intent Mapping
        </div>
        <div class="flex flex-wrap gap-2 p-3 bg-bg-main border border-border-main rounded-xl">
            {#each data.intents as intent}
                <span class="text-[9px] font-mono font-bold bg-primary-blue/5 text-primary-blue px-2.5 py-1 rounded border border-primary-blue/20">
                    {intent}
                </span>
            {:else}
                <span class="text-[9px] font-mono text-text-muted italic px-2 py-1">Auto-mapping GENERAL scope...</span>
            {/each}
        </div>
    </div>

    <!-- Description -->
    <div class="space-y-2">
        <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-[0.2em] text-accent-emerald/60">
            <Info class="w-3.5 h-3.5" />
            Functional Objective
        </div>
        <div class="p-4 bg-bg-main border border-border-main rounded-xl text-sm leading-relaxed text-neutral-300">
            {data.description}
        </div>
    </div>

    <!-- How it Works / Roadmap -->
    <div class="space-y-2">
        <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-[0.2em] text-primary-blue/60">
            <Map class="w-3.5 h-3.5" />
            Architectural Roadmap
        </div>
        <div class="p-4 bg-bg-main border border-border-main rounded-xl text-xs font-mono leading-relaxed text-neutral-400 whitespace-pre-wrap">
            {data.how_it_works}
        </div>
    </div>

    <!-- Parameter Schema -->
    <div class="space-y-2">
        <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-[0.2em] text-amber-500/60">
            <Brackets class="w-3.5 h-3.5" />
            Input Parameter Schema
        </div>
        <div class="p-4 bg-[#0d1117] border border-border-main rounded-xl overflow-x-auto custom-scrollbar">
            <pre class="text-[11px] font-mono text-amber-400/90 leading-tight">
                {JSON.stringify(data.schema_json, null, 2)}
            </pre>
        </div>
    </div>

    <div class="mt-4 pt-4 border-t border-border-main flex items-center gap-3">
        <div class="p-2 bg-emerald-500/5 rounded-lg border border-emerald-500/10">
            <Terminal class="w-4 h-4 text-emerald-500" />
        </div>
        <p class="text-[10px] text-text-muted leading-snug italic">
            Review the blueprint carefully. Once approved, the SWE Agent will begin autonomous synthesis and recursive validation.
        </p>
    </div>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
        height: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: #1e293b;
        border-radius: 10px;
    }
</style>
