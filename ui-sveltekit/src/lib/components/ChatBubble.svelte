<script lang="ts">
    import {onMount} from 'svelte';
    import {ChevronDown, ChevronRight, Cpu} from 'lucide-svelte';
    import {mdIt} from "$lib/utils";

    let { content = '', thought = '' } = $props();

    let renderedContent = $state('');
    let renderedThought = $state('');

    let thoughtExpanded = $state(false);

    async function init() {

        render();
    }

    function render() {
        if (mdIt) {
            renderedContent = mdIt.render(content);
            if (thought) {
                const cleanThought = thought.replace(/<\/?thinking>/g, '');
                renderedThought = mdIt.render(cleanThought);
            }
        }
    }

    onMount(() => {
        init();
        
        if (typeof window !== 'undefined' && !(window as any).copyToClipboard) {
            (window as any).copyToClipboard = (btn: HTMLButtonElement) => {
                const code = decodeURIComponent(btn.getAttribute('data-code') || '');
                navigator.clipboard.writeText(code).then(() => {
                    const originalInner = btn.innerHTML;
                    btn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="text-emerald-500"><polyline points="20 6 9 17 4 12"/></svg>`;
                    btn.classList.add('copied');
                    setTimeout(() => {
                        btn.innerHTML = originalInner;
                        btn.classList.remove('copied');
                    }, 2000);
                });
            };
        }
    });

    $effect(() => {
        if (mdIt && (content || thought)) {
            render();
        }
    });
</script>

<div class="flex flex-col space-y-4">
    {#if thought}
        <div class="relative group/thought">
            <button 
                onclick={() => thoughtExpanded = !thoughtExpanded}
                class="flex items-center gap-2 mb-2 text-[9px] font-bold text-zinc-600 uppercase tracking-widest hover:text-zinc-400 transition-colors"
            >
                <Cpu class="w-3 h-3"/>
                Deep Thought
                {#if thoughtExpanded}
                    <ChevronDown class="w-3 h-3" />
                {:else}
                    <ChevronRight class="w-3 h-3" />
                {/if}
            </button>
            
            {#if thoughtExpanded}
                <div class="p-4 bg-zinc-900/30 border-l-2 border-zinc-700 rounded-r-lg text-xs text-zinc-500 font-mono italic leading-relaxed prose prose-invert prose-sm max-w-none animate-in fade-in slide-in-from-top-1 duration-200">
                    {@html renderedThought}
                </div>
            {/if}
        </div>
    {/if}

    <div class="prose prose-invert max-w-none prose-sm text-zinc-200">
        {@html renderedContent}
    </div>
</div>

<style>
    :global(.prose pre) {
        background-color: #0c0c0e !important;
        padding: 1rem;
        border-radius: 0.5rem;
        border: 1px solid #27272a;
        margin: 1rem 0;
        white-space: pre-wrap;
        word-break: break-all;
        position: relative;
    }
    :global(.prose pre code) {
        white-space: pre-wrap;
    }
    :global(.prose :not(pre) > code) {
        background-color: #18181b;
        padding: 0.2rem 0.4rem;
        border-radius: 0.25rem;
        color: #e4e4e7;
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    }

    :global(.copy-button) {
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        padding: 0.25rem;
        border-radius: 0.375rem;
        background-color: rgba(24, 24, 27, 0.8);
        border: 1px solid #3f3f46;
        color: #a1a1aa;
        opacity: 0;
        transition: all 0.2s;
        cursor: pointer;
        z-index: 10;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    :global(.prose pre:hover .copy-button) {
        opacity: 1;
    }

    :global(.copy-button:hover) {
        background-color: #3f3f46;
        color: #f4f4f5;
        border-color: #52525b;
    }

    :global(.copy-button.copied) {
        opacity: 1;
        border-color: #10b981;
    }
</style>
