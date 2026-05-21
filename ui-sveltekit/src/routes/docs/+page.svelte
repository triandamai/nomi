<script lang="ts">
    import { onMount, tick } from 'svelte';
    import { chatApi } from '$lib/api/client';
    import { mdIt, setupMarkdownHelpers } from '$lib/utils';
    import { FileText, Loader2, BookOpen, ChevronRight, Hash } from 'lucide-svelte';

    let content = $state('');
    let renderedContent = $state('');
    let isLoading = $state(true);

    async function fetchDoc() {
        isLoading = true;
        try {
            const res = await chatApi.getDocumentation();
            content = res.data;
            if (mdIt) {
                renderedContent = mdIt.render(content);
                
                // Initialize mermaid after rendering
                await tick();
                try {
                    const mermaid = (await import('mermaid')).default;
                    mermaid.initialize({
                        startOnLoad: false,
                        theme: 'dark',
                        securityLevel: 'loose',
                        fontFamily: 'inherit',
                    });
                    
                    const nodes = document.querySelectorAll('.mermaid');
                    if (nodes.length > 0) {
                        await mermaid.run({
                            nodes: Array.from(nodes) as HTMLElement[],
                        });
                    }
                } catch (err) {
                    console.error("Mermaid initialization failed:", err);
                }
            }
        } catch (e) {
            console.error(e);
        } finally {
            isLoading = false;
        }
    }

    onMount(() => {
        setupMarkdownHelpers();
        fetchDoc();
    });
</script>

<div class="flex flex-col h-screen bg-slate-950 text-slate-200">
    <main class="flex-1 overflow-y-auto custom-scrollbar">
        <div class="max-w-5xl mx-auto px-6 py-12">
            {#if isLoading}
                <div class="flex flex-col items-center justify-center h-[60vh] space-y-4">
                    <Loader2 class="w-12 h-12 text-sky-500 animate-spin" />
                    <p class="text-slate-500 font-black uppercase tracking-[0.3em] text-xs">Fetching System Spec...</p>
                </div>
            {:else}
                <div class="bg-slate-900/40 border border-slate-800 rounded-2xl overflow-hidden shadow-2xl">
                    <!-- Doc Header -->
                    <div class="p-8 border-b border-slate-800 bg-gradient-to-br from-slate-900 to-slate-950">
                        <div class="flex items-center gap-4 mb-4">
                            <div class="p-3 bg-sky-500/10 rounded-xl border border-sky-500/20">
                                <BookOpen class="w-6 h-6 text-sky-400" />
                            </div>
                            <div>
                                <h1 class="text-2xl font-black text-white uppercase tracking-tighter">Nomi System Blueprint</h1>
                                <p class="text-sm text-slate-400 italic">Technical Specification Document (TSD)</p>
                            </div>
                        </div>
                    </div>

                    <!-- Rendered Markdown -->
                    <div class="p-8 lg:p-12 prose prose-invert prose-sky max-w-none">
                        {@html renderedContent}
                    </div>
                </div>

                <!-- Footer -->
                <div class="mt-8 flex items-center justify-between text-[10px] text-slate-600 font-mono uppercase tracking-widest px-4">
                    <span>Generated from root/README.md</span>
                    <span>System Version 2.0.0-FLASH</span>
                </div>
            {/if}
        </div>
    </main>
</div>

<style>
    .custom-scrollbar::-webkit-scrollbar {
        width: 6px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: #020617;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: #1e293b;
        border-radius: 10px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: #334155;
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
        margin: 2rem 0;
        backdrop-filter: blur(4px);
        box-shadow: 0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1);
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

    /* Premium Typography Overrides */
    :global(.prose h1) {
        font-size: 2.25rem !important; /* 4xl */
        font-weight: 900 !important;
        text-transform: uppercase !important;
        letter-spacing: -0.05em !important;
        margin-top: 1rem !important;
        margin-bottom: 3rem !important;
        color: #ffffff !important;
    }

    :global(.prose h2) {
        font-size: 1.5rem !important; /* 2xl */
        font-weight: 900 !important;
        text-transform: uppercase !important;
        letter-spacing: -0.025em !important;
        border-bottom: 1px solid #1e293b !important;
        padding-bottom: 0.75rem !important;
        margin-top: 4rem !important;
        margin-bottom: 2rem !important;
        color: #ffffff !important;
    }

    :global(.prose h3) {
        font-size: 1.125rem !important; /* lg */
        font-weight: 900 !important;
        text-transform: uppercase !important;
        margin-top: 2.5rem !important;
        margin-bottom: 1.5rem !important;
        color: #38bdf8 !important; /* sky-400 */
    }

    :global(.prose p) {
        color: #cbd5e1 !important; /* slate-300 */
        line-height: 1.75 !important;
        margin-bottom: 1.5rem !important;
    }

    :global(.prose code) {
        color: #38bdf8 !important;
        background-color: #020617 !important;
        padding: 0.125rem 0.375rem !important;
        border-radius: 0.25rem !important;
        font-size: 0.875em !important;
    }

    :global(.prose code::before), :global(.prose code::after) {
        content: "" !important;
    }

    :global(.prose pre) {
        background-color: #020617 !important;
        padding: 1rem !important;
        border-radius: 1rem !important;
        border: 1px solid #1e293b !important;
        margin: 1.5rem 0 !important;
        white-space: pre-wrap !important;
        word-break: break-all !important;
        position: relative !important;
    }

    :global(.prose pre code) {
        white-space: pre-wrap !important;
    }

    :global(.prose pre.collapsed code) {
        display: none !important;
    }

    :global(.prose pre.collapsed) {
        padding-top: 2.5rem !important;
        padding-bottom: 0rem !important;
        min-height: 2.5rem !important;
    }

    :global(.code-block-header) {
        position: absolute !important;
        top: 0 !important;
        left: 0 !important;
        right: 0 !important;
        height: 2.5rem !important;
        display: flex !important;
        align-items: center !important;
        justify-content: space-between !important;
        padding: 0 1rem !important;
        background-color: #1e293b !important;
        border-bottom: 1px solid #334155 !important;
        border-top-left-radius: 1rem !important;
        border-top-right-radius: 1rem !important;
        z-index: 10 !important;
    }

    :global(.code-lang) {
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace !important;
        font-size: 10px !important;
        font-weight: 700 !important;
        text-transform: uppercase !important;
        letter-spacing: 0.1em !important;
        color: #94a3b8 !important;
    }

    :global(.code-header-actions) {
        display: flex !important;
        align-items: center !important;
        gap: 0.5rem !important;
    }

    :global(.copy-button), :global(.toggle-button) {
        padding: 0.25rem !important;
        border-radius: 0.375rem !important;
        background-color: transparent !important;
        border: 1px solid transparent !important;
        color: #64748b !important;
        transition: all 0.2s !important;
        cursor: pointer !important;
        display: flex !important;
        align-items: center !important;
        justify-content: center !important;
    }

    :global(.copy-button:hover), :global(.toggle-button:hover) {
        background-color: #334155 !important;
        color: #f1f5f9 !important;
        border-color: #475569 !important;
    }

    :global(.copy-button.copied) {
        color: #10b981 !important;
    }

    :global(.prose pre.collapsed .toggle-icon) {
        transform: rotate(-90deg) !important;
    }

    :global(.toggle-icon) {
        transition: transform 0.2s !important;
    }

    /* Custom List Styling */
    :global(.prose ul) {
        list-style-type: none !important;
        padding-left: 0 !important;
    }

    :global(.prose ul li) {
        position: relative;
        padding-left: 1.75rem;
        margin-bottom: 0.5rem;
    }

    :global(.prose ul li::before) {
        content: "◆";
        position: absolute;
        left: 0.25rem;
        color: #38bdf8; /* sky-400 */
        font-size: 0.7rem;
        top: 0.125rem;
        opacity: 0.8;
    }

    /* Nested List Indentation & Markers */
    :global(.prose ul ul) {
        margin-top: 0.5rem !important;
        margin-bottom: 0.5rem !important;
    }

    :global(.prose ul ul li) {
        padding-left: 1.5rem;
    }

    :global(.prose ul ul li::before) {
        content: "◇";
        font-size: 0.8rem;
        opacity: 0.6;
    }

    :global(.prose ul ul ul li::before) {
        content: "▪";
        font-size: 0.6rem;
        left: 0.125rem;
        opacity: 0.5;
    }

    :global(.prose ol) {
        counter-reset: item;
        padding-left: 0 !important;
        list-style-type: none !important;
    }

    :global(.prose ol li) {
        display: block;
        position: relative;
        padding-left: 1.75rem;
        margin-bottom: 0.5rem;
    }

    :global(.prose ol li::before) {
        content: counter(item) ".";
        counter-increment: item;
        position: absolute;
        left: 0;
        font-weight: 900;
        color: #38bdf8;
        font-family: ui-monospace, monospace;
        font-size: 0.75rem;
        top: 0.125rem;
    }

    /* Mixed Nesting Support */
    :global(.prose ul ol), :global(.prose ol ul), :global(.prose ol ol), :global(.prose ul ul) {
        margin-top: 0.5rem !important;
        margin-bottom: 0.5rem !important;
        padding-left: 0.5rem !important;
    }

    /* Ensure markers are reset correctly in mixed nests */
    :global(.prose ol ul li::before) {
        content: "◇";
        font-size: 0.8rem;
    }

    :global(.prose ul ol li::before) {
        content: counter(item) ".";
        counter-increment: item;
        font-family: ui-monospace, monospace;
    }

    :global(.prose ol ol li::before) {
        content: counter(item, lower-alpha) ".";
        font-weight: 700;
        font-size: 0.7rem;
    }

    :global(.prose ol ol ol li::before) {
        content: counter(item, lower-roman) ".";
        font-size: 0.65rem;
    }
</style>
