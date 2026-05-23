<script lang="ts">
    import {onMount, tick} from 'svelte';
    import {ChevronDown, ChevronRight, Cpu, ExternalLink, FileText, Play, Music, Eye, Factory, Sparkles, ArrowRight} from 'lucide-svelte';
    import {mdIt, formatDate, setupMarkdownHelpers} from "$lib/utils";
    import {env} from '$env/dynamic/public';
    import {goto} from '$app/navigation';

    let {
        content = '',
        thought = '',
        image_url = '',
        video_url = '',
        audio_url = '',
        document_url = '',
        sticker_url = '',
        metadata = {},
        onToggleThought = () => {}
    } = $props();

    const BASE_URL = env.PUBLIC_GATEWAY_URL || 'http://localhost:8000/api';
    const FILE_URL = BASE_URL.replace('/api', '') + '/api/files/';

    let renderedContent = $state('');
    let renderedThought = $state('');
    let mediaContext = $state('');

    let thoughtExpanded = $state(false);
    let contextExpanded = $state(false);

    function toggleThought() {
        thoughtExpanded = !thoughtExpanded;
        onToggleThought(thoughtExpanded);
    }

    async function init() {
        render();
    }

    let bubbleElement = $state<HTMLElement | null>(null);

    async function render() {
        if (mdIt) {
            let displayContent = content;
            
            // Extract [Media Context: ...]
            const mediaContextRegex = /\[Media Context: (.*?)\] /s;
            const match = displayContent.match(mediaContextRegex);
            
            if (match) {
                mediaContext = match[1];
                displayContent = displayContent.replace(mediaContextRegex, '');
            } else {
                // Fallback for cases without trailing space
                const mediaContextRegexNoSpace = /\[Media Context: (.*?)\]/s;
                const matchNoSpace = displayContent.match(mediaContextRegexNoSpace);
                if (matchNoSpace) {
                    mediaContext = matchNoSpace[1];
                    displayContent = displayContent.replace(mediaContextRegexNoSpace, '');
                } else {
                    mediaContext = '';
                }
            }

            renderedContent = mdIt.render(displayContent);
            
            if (thought) {
                const cleanThought = thought.replace(/<\/?thinking>/g, '');
                renderedThought = mdIt.render(cleanThought);
            }

            // If mermaid containers exist, initialize them
            if (renderedContent.includes('class="mermaid"')) {
                await tick();
                try {
                    const mermaid = (await import('mermaid')).default;
                    mermaid.initialize({
                        startOnLoad: false,
                        theme: 'dark',
                        securityLevel: 'loose',
                        fontFamily: 'inherit',
                    });
                    
                    if (bubbleElement) {
                        const nodes = bubbleElement.querySelectorAll('.mermaid');
                        if (nodes.length > 0) {
                            await mermaid.run({
                                nodes: Array.from(nodes) as HTMLElement[],
                            });
                        }
                    }
                } catch (e) {
                    console.error("Mermaid in chat failed:", e);
                }
            }
        }
    }


    onMount(() => {
        init();
        setupMarkdownHelpers();
    });

    $effect(() => {
        if (mdIt && (content || thought)) {
            render();
        }
    });
</script>

<div bind:this={bubbleElement} class="flex flex-col space-y-4">
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

        {#if sticker_url}
            <div class="mb-4 rounded-xl overflow-hidden group/sticker relative w-fit">
                <img
                        src={sticker_url.startsWith("http") ? sticker_url : FILE_URL + sticker_url}
                        alt="Sticker"
                        class="max-w-[150px] h-auto object-contain"
                />
            </div>
        {/if}

        {#if video_url}
            <div class="mb-4 rounded-xl overflow-hidden border border-slate-800 bg-slate-900/50 group/video relative max-w-lg">
                <video
                        src={video_url.startsWith("http") ? video_url : FILE_URL + video_url}
                        controls
                        class="w-full h-auto max-h-[400px]"
                >
                    <track kind="captions" />
                </video>
            </div>
        {/if}

        {#if audio_url}
            <div class="mb-4 p-3 rounded-xl border border-slate-800 bg-slate-900/50 flex items-center gap-4 max-w-sm">
                <div class="w-10 h-10 rounded-full bg-blue-500/20 flex items-center justify-center shrink-0 text-blue-400">
                    <Music class="w-5 h-5"/>
                </div>
                <div class="flex-1 min-w-0">
                    <div class="text-[10px] font-bold text-slate-500 uppercase tracking-widest mb-1">Voice Note / Audio</div>
                    <audio
                            src={audio_url.startsWith("http") ? audio_url : FILE_URL + audio_url}
                            controls
                            class="w-full h-8"
                    ></audio>
                </div>
            </div>
        {/if}

        {#if document_url}
            <a
                    href={document_url.startsWith("http") ? document_url : FILE_URL + document_url}
                    target="_blank"
                    class="mb-4 p-4 rounded-xl border border-slate-800 bg-slate-900/50 flex items-center gap-4 max-w-md hover:bg-slate-800/80 transition-all group/doc"
            >
                <div class="w-12 h-12 rounded-lg bg-emerald-500/20 flex items-center justify-center shrink-0 text-emerald-400 group-hover/doc:scale-110 transition-transform">
                    <FileText class="w-6 h-6"/>
                </div>
                <div class="flex-1 min-w-0">
                    <div class="text-[10px] font-bold text-slate-500 uppercase tracking-widest mb-0.5">Attached Document</div>
                    <div class="text-sm font-medium text-slate-200 truncate">{document_url.split('/').pop()}</div>
                </div>
                <ExternalLink class="w-4 h-4 text-slate-500 group-hover/doc:text-emerald-400"/>
            </a>
        {/if}

        {#if mediaContext}
            <div class="relative group/context mb-4">
                <button
                        onclick={() => contextExpanded = !contextExpanded}
                        class="flex items-center gap-2 mb-2 text-[9px] font-black text-slate-500 uppercase tracking-widest hover:text-emerald-400 transition-colors"
                >
                    <Eye class="w-3 h-3"/>
                    Visual Context
                    {#if contextExpanded}
                        <ChevronDown class="w-3 h-3"/>
                    {:else}
                        <ChevronRight class="w-3 h-3"/>
                    {/if}
                </button>

                {#if contextExpanded}
                    <div class="p-4 bg-slate-900/20 border-l-2 border-emerald-500/30 rounded-r-lg text-xs text-slate-400 font-mono italic leading-relaxed animate-in fade-in slide-in-from-top-1 duration-200">
                        {mediaContext}
                    </div>
                {/if}
            </div>
        {/if}

        {#if metadata?.proposal_slug}
            <div class="mb-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
                <div class="bg-border-main/20 border border-border-main rounded-2xl overflow-hidden shadow-2xl backdrop-blur-sm group/proposal hover:border-accent-emerald/40 transition-all duration-300">
                    <div class="px-5 py-3 border-b border-border-main bg-border-main/40 flex items-center justify-between">
                        <div class="flex items-center gap-2 text-[10px] font-black uppercase tracking-widest text-accent-emerald">
                            <Factory class="w-3.5 h-3.5" />
                            Autonomous Blueprint Proposed
                        </div>
                        <Sparkles class="w-3.5 h-3.5 text-accent-emerald animate-pulse" />
                    </div>
                    <div class="p-5 flex flex-col gap-4">
                        <div class="flex items-center gap-4">
                            <div class="p-3 bg-bg-main rounded-xl border border-border-main group-hover/proposal:border-accent-emerald/20 transition-colors">
                                <Cpu class="w-6 h-6 text-accent-emerald opacity-80" />
                            </div>
                            <div class="flex-1 min-w-0">
                                <p class="text-[10px] font-bold text-text-muted uppercase tracking-widest mb-1">Target Skill Handle</p>
                                <h4 class="text-sm font-mono font-bold text-white truncate">{metadata.proposal_slug}</h4>
                            </div>
                        </div>

                        <button 
                            onclick={() => goto(`/dashboard/srp/proposals`)}
                            class="w-full py-3 bg-accent-emerald hover:bg-accent-emerald/80 text-bg-main font-black rounded-xl text-[10px] uppercase tracking-[0.2em] transition-all flex items-center justify-center gap-2 shadow-lg shadow-accent-emerald/10 active:scale-[0.98]"
                        >
                            <span>Open Factory Console</span>
                            <ArrowRight class="w-3.5 h-3.5" />
                        </button>
                    </div>
                </div>
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

    :global(.prose pre.collapsed code) {
        display: none;
    }

    :global(.prose pre.collapsed) {
        padding-top: 2.5rem;
        padding-bottom: 0rem;
        min-height: 2.5rem;
    }

    :global(.code-block-header) {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 2.5rem;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 1rem;
        background-color: #1e293b;
        border-bottom: 1px solid #334155;
        border-top-left-radius: 1rem;
        border-top-right-radius: 1rem;
    }

    :global(.code-lang) {
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
        font-size: 10px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: #94a3b8;
    }

    :global(.code-header-actions) {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    :global(.copy-button), :global(.toggle-button) {
        padding: 0.25rem;
        border-radius: 0.375rem;
        background-color: transparent;
        border: 1px solid transparent;
        color: #64748b;
        transition: all 0.2s;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    :global(.copy-button:hover), :global(.toggle-button:hover) {
        background-color: #334155;
        color: #f1f5f9;
        border-color: #475569;
    }

    :global(.copy-button.copied) {
        color: #10b981;
    }

    :global(.prose pre.collapsed .toggle-icon) {
        transform: rotate(-90deg);
    }

    :global(.toggle-icon) {
        transition: transform 0.2s;
    }

    :global(.mermaid-container) {
        max-width: 100%;
        overflow-x: auto;
        display: flex;
        justify-content: center;
        background: rgba(15, 23, 42, 0.4);
        border: 1px solid rgba(51, 65, 85, 0.5);
        border-radius: 1rem;
        padding: 1.5rem;
        margin: 1.5rem 0;
        backdrop-filter: blur(4px);
    }

    :global(.mermaid-container pre.mermaid) {
        background: transparent !important;
        border: none !important;
        padding: 0 !important;
        margin: 0 !important;
        display: flex;
        justify-content: center;
        width: 100%;
    }

    :global(.mermaid) {
        background-color: transparent !important;
        display: flex;
        justify-content: center;
        width: 100%;
    }

    :global(.mermaid svg) {
        max-width: 100% !important;
        height: auto !important;
    }
</style>
