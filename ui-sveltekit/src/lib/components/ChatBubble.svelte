<script lang="ts">
    import {onMount} from 'svelte';
    import {ChevronDown, ChevronRight, Cpu, ExternalLink, FileText, Play, Music, Eye} from 'lucide-svelte';
    import {mdIt, formatDate} from "$lib/utils";
    import {env} from '$env/dynamic/public';

    let {
        content = '',
        thought = '',
        image_url = '',
        video_url = '',
        audio_url = '',
        document_url = '',
        sticker_url = '',
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

    function render() {
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
        }
    }


    onMount(() => {
        init();

        if (typeof window !== 'undefined') {
            if (!(window as any).copyToClipboard) {
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

            if (!(window as any).toggleCodeBlock) {
                (window as any).toggleCodeBlock = (btn: HTMLButtonElement) => {
                    const pre = btn.closest('pre');
                    if (pre) {
                        pre.classList.toggle('collapsed');
                    }
                };
            }
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
</style>
