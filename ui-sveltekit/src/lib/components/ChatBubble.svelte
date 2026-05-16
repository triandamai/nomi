<script lang="ts">
    import {onMount} from 'svelte';
    import {ChevronDown, ChevronRight, Cpu, ExternalLink} from 'lucide-svelte';
    import {mdIt, formatDate} from "$lib/utils";
    import {env} from '$env/dynamic/public';

    let {content = '', thought = '', image_url = '', created_at = '', onToggleThought = () => {}} = $props();

    const BASE_URL = env.PUBLIC_GATEWAY_URL || 'http://localhost:8000/api';
    const FILE_URL = BASE_URL.replace('/api', '') + '/api/files/';

    let renderedContent = $state('');
    let renderedThought = $state('');

    let thoughtExpanded = $state(false);

    function toggleThought() {
        thoughtExpanded = !thoughtExpanded;
        onToggleThought(thoughtExpanded);
    }

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
                    onclick={toggleThought}
                    class="flex items-center gap-2 mb-2 text-[9px] font-black text-slate-500 uppercase tracking-widest hover:text-blue-400 transition-colors"
            >
                <Cpu class="w-3 h-3"/>
                Deep Thought
                {#if thoughtExpanded}
                    <ChevronDown class="w-3 h-3"/>
                {:else}
                    <ChevronRight class="w-3 h-3"/>
                {/if}
            </button>

            {#if thoughtExpanded}
                <div class="p-4 bg-slate-900/30 border-l-2 border-blue-500/50 rounded-r-lg text-xs text-slate-500 font-mono italic leading-relaxed prose prose-invert prose-sm max-w-none animate-in fade-in slide-in-from-top-1 duration-200">
                    {@html renderedThought}
                </div>
            {/if}
        </div>
    {/if}

    <div class="prose prose-invert max-w-none prose-sm text-slate-200">
        {#if image_url}
            <div class="mb-4 rounded-xl overflow-hidden border border-slate-800 bg-slate-900/50 group/image relative w-fit">
                <img
                        src={image_url.startsWith("http") ? image_url : FILE_URL + image_url}
                        alt="Uploaded content"
                        class="max-w-full h-auto max-h-[350px] object-contain"
                        onerror={(e) => {
                            const img = e.currentTarget as HTMLImageElement;
                            img.src = 'https://placehold.co/600x400/18181b/a1a1aa?text=Image+Load+Failed';
                            img.className = "max-w-full h-auto opacity-50 grayscale";
                        }}
                />
                <a
                        href={image_url.startsWith("http") ? image_url : FILE_URL + image_url}
                        target="_blank"
                        class="absolute top-2 right-2 p-2 bg-slate-900/80 rounded-lg opacity-0 group-hover/image:opacity-100 transition-opacity hover:text-emerald-400"
                        title="Open full size"
                >
                    <ExternalLink class="w-4 h-4"/>
                </a>
            </div>
        {/if}
        {@html renderedContent}
    </div>
</div>

<style>
    :global(.prose pre) {
        background-color: #020617 !important;
        padding: 1rem;
        border-radius: 1rem;
        border: 1px solid #1e293b;
        margin: 1rem 0;
        white-space: pre-wrap;
        word-break: break-all;
        position: relative;
    }

    :global(.prose pre code) {
        white-space: pre-wrap;
    }

    :global(.prose :not(pre) > code) {
        background-color: #1e293b;
        padding: 0.2rem 0.4rem;
        border-radius: 0.25rem;
        color: #f1f5f9;
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    }

    :global(.copy-button) {
        position: absolute;
        top: 0.5rem;
        right: 0.5rem;
        padding: 0.25rem;
        border-radius: 0.5rem;
        background-color: rgba(15, 23, 42, 0.8);
        border: 1px solid #334155;
        color: #64748b;
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
        background-color: #334155;
        color: #f1f5f9;
        border-color: #475569;
    }

    :global(.copy-button.copied) {
        opacity: 1;
        border-color: #3b82f6;
    }
</style>
