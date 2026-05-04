<script lang="ts">
    import {CheckCircle2, Code, Database, Globe, Loader2} from 'lucide-svelte';


    let {tool,result,args} = $props<{tool:string,args:string,result:string|undefined}>()


    const getIcon = (name: string) => {
        if (name.includes('file')) return Code;
        if (name.includes('sql')) return Database;
        if (name.includes('web')) return Globe;
        return CheckCircle2;
    };

    const Icon = getIcon(tool.tool);
</script>

<div class="my-4 border border-zinc-800 rounded-lg overflow-hidden bg-zinc-900/30">
    <div class="flex items-center justify-between px-4 py-2 border-b border-zinc-800 bg-zinc-900/50">
        <div class="flex items-center gap-2">
            <Icon class="w-4 h-4 text-zinc-400" />
            <span class="text-xs font-mono text-zinc-300">{tool.tool}</span>
        </div>
        {#if !result}
            <div class="flex items-center gap-2 text-[10px] text-zinc-500 uppercase tracking-widest animate-pulse">
                <Loader2 class="w-3 h-3 animate-spin" />
                Executing...
            </div>
        {:else}
            <div class="flex items-center gap-1 text-[10px] text-emerald-500 uppercase tracking-widest font-bold">
                <CheckCircle2 class="w-3 h-3" />
                Completed
            </div>
        {/if}
    </div>

    <div class="p-3">
        <div class="mb-2">
            <div class="text-[10px] text-zinc-500 uppercase mb-1">Arguments</div>
            <pre class="text-[11px] font-mono text-zinc-400 bg-black/20 p-2 rounded">
                {JSON.stringify(args, null, 2)}
            </pre>
        </div>

        {#if result}
            <div>
                <div class="text-[10px] text-zinc-500 uppercase mb-1">Result</div>
                <div class="max-h-60 overflow-y-auto">
                    {#if tool === 'execute_sql_query'}
                        <pre class="text-[11px] font-mono text-emerald-400/80 bg-black/40 p-2 rounded">{result}</pre>
                    {:else if tool === 'read_workspace_file'}
                        <pre class="text-[11px] font-mono text-blue-400/80 bg-black/40 p-2 rounded">{result}</pre>
                    {:else}
                        <div class="text-xs text-zinc-300 leading-relaxed bg-black/20 p-2 rounded whitespace-pre-wrap">{result}</div>
                    {/if}
                </div>
            </div>
        {/if}
    </div>
</div>
