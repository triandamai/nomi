<script lang="ts">
    import {onMount, tick, untrack} from 'svelte';
    import {
        ChevronDown,
        ChevronRight,
        Cpu,
        ExternalLink,
        FileText,
        Play,
        Music,
        Eye,
        Factory,
        Sparkles,
        ArrowRight,
        CornerUpLeft
    } from 'lucide-svelte';
    import {mdIt, formatDate, setupMarkdownHelpers, useAvatar} from "$lib/utils";
    import {env} from '$env/dynamic/public';
    import {goto} from '$app/navigation';
    import ReminderCard from './ReminderCard.svelte';
    import FinanceCard from './FinanceCard.svelte';
    import PluginProposalCard from './PluginProposalCard.svelte';
    import TaskCard from './TaskCard.svelte';
    import { chatStore } from '$lib/stores/chat.svelte';
    import {mentionStore} from '$lib/stores/mentions.svelte';


    let {
        content = '',
        thought = '',
        image_url = '',
        video_url = '',
        audio_url = '',
        document_url = '',
        sticker_url = '',
        metadata = {},
        replied_message = null,
        onToggleThought = () => {}
    } = $props();

    const BASE_URL = env.PUBLIC_GATEWAY_URL || 'http://localhost:8000/api';
    const FILE_URL = BASE_URL.replace('/api', '') + '/api/files/';

    let renderedContent = $state('');
    let renderedThought = $state('');
    let mediaContext = $state('');

    let thoughtExpanded = $state(false);
    let contextExpanded = $state(false);

    let formValues = $state<Record<string, string>>({});
    let formSubmitted = $state(false);

    let requiredFields = $derived.by(() => {
        const fields: any[] = [];
        const regex = /<\s*RequiredField\s+name="([^"]+)"\s+label="([^"]+)"\s+type="([^"]+)"(?:\s+options="([^"]+)")?\s*\/>/gi;
        let match;
        while ((match = regex.exec(content)) !== null) {
            fields.push({
                name: match[1],
                label: match[2],
                type: match[3],
                options: match[4] ? match[4].split(',') : []
            });
        }
        return fields;
    });

    async function submitForm(key?: string, value?: string) {
        if (formSubmitted) return;

        let replyText = "";
        if (key && value) {
            formValues[key] = value;
            replyText = `${value}`;
        } else {
            replyText = Object.entries(formValues)
                .map(([k, v]) => `${k}: ${v}`)
                .join(', ');
        }

        if (!replyText) return;

        formSubmitted = true;
        await chatStore.sendMessage(replyText);
    }

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
            let displayContent = content.replace(/<\s*RequiredField[^>]*\/>/gi, '');
            
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
            
            // Wrap mentions (WhatsApp/Telegram numbers, platform UUIDs or usernames) in a premium mention-pill
            const mentionRegex = /@([a-zA-Z0-9_\-]+)\b/g;
            renderedContent = renderedContent.replace(mentionRegex, '<span class="mention-pill" data-external-id="$1">@$1</span>');
            
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


    let parsedMetadata = $derived.by(() => {
        if (!metadata) return {};
        if (typeof metadata === 'string') {
            try {
                return JSON.parse(metadata);
            } catch (e) {
                console.error("ChatBubble: Failed to parse metadata string:", metadata);
                return {};
            }
        }
        return metadata;
    });

    onMount(() => {
        init();
        setupMarkdownHelpers();
    });

    $effect(() => {
        // Explicitly track content, thought, and mdIt
        const _trigger = { content, thought, mdIt };

        if (mdIt && (content || thought)) {
            untrack(() => {
                render();
            });
        }
    });

    // Tooltip hover state
    let hoverTooltip = $state<{
        visible: boolean;
        x: number;
        y: number;
        displayName: string;
        externalId: string;
        avatarUrl: string;
    }>({
        visible: false,
        x: 0,
        y: 0,
        displayName: '',
        externalId: '',
        avatarUrl: ''
    });

    // Reactive mention display name sync and hover listeners
    $effect(() => {
        if (renderedContent && bubbleElement) {
            // 1. Synchronously read the cache for all matched mention IDs
            // This forces Svelte to track them as dependencies of this effect!
            const mentionRegex = /@([a-zA-Z0-9_\-]+)\b/g;
            const matches = [...renderedContent.matchAll(mentionRegex)];
            const resolvedNames: Record<string, string> = {};
            for (const match of matches) {
                const extId = match[1];
                resolvedNames[extId] = mentionStore.getDisplayName(extId);
            }

            let activePills: HTMLElement[] = [];

            // 2. Schedule the DOM update after the tick
            tick().then(() => {
                if (!bubbleElement) return;
                const pills = bubbleElement.querySelectorAll('.mention-pill');
                activePills = Array.from(pills) as HTMLElement[];

                activePills.forEach((pill) => {
                    const extId = pill.getAttribute('data-external-id');
                    if (extId && resolvedNames[extId]) {
                        const displayName = resolvedNames[extId];
                        
                        // Update text content reactively
                        pill.textContent = displayName.startsWith('@') ? displayName : `@${displayName}`;

                        // Clean up any old listeners attached previously
                        if ((pill as any)._onMouseEnter) {
                            pill.removeEventListener('mouseenter', (pill as any)._onMouseEnter);
                        }
                        if ((pill as any)._onMouseLeave) {
                            pill.removeEventListener('mouseleave', (pill as any)._onMouseLeave);
                        }

                        // Tooltip hover events
                        const onMouseEnter = (e: MouseEvent) => {
                            const rect = pill.getBoundingClientRect();
                            hoverTooltip = {
                                visible: true,
                                x: rect.left + rect.width / 2,
                                y: rect.top - 10,
                                displayName: displayName.startsWith('@') ? displayName.substring(1) : displayName,
                                externalId: extId,
                                avatarUrl: useAvatar(displayName)
                            };
                        };

                        const onMouseLeave = () => {
                            hoverTooltip.visible = false;
                        };

                        pill.addEventListener('mouseenter', onMouseEnter);
                        pill.addEventListener('mouseleave', onMouseLeave);

                        // Cache listener functions on the element for clean removal
                        (pill as any)._onMouseEnter = onMouseEnter;
                        (pill as any)._onMouseLeave = onMouseLeave;
                    }
                });
            });

            // 3. Synchronous cleanup of current listeners when effect is destroyed/re-run
            return () => {
                activePills.forEach((pill) => {
                    if ((pill as any)._onMouseEnter) {
                        pill.removeEventListener('mouseenter', (pill as any)._onMouseEnter);
                    }
                    if ((pill as any)._onMouseLeave) {
                        pill.removeEventListener('mouseleave', (pill as any)._onMouseLeave);
                    }
                });
            };
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
        {#if replied_message || parsedMetadata?.quoted_message}
            {@const q = replied_message || parsedMetadata.quoted_message}
            <div class="mb-3 animate-in fade-in slide-in-from-top-1 duration-300">
                <div class="bg-slate-800/40 border-l-4 border-blue-500/50 rounded-r-xl p-3 flex flex-col gap-1 hover:bg-slate-800/60 transition-all cursor-default group/quote">
                    <div class="flex items-center gap-2 text-[9px] font-black uppercase tracking-widest text-blue-400/80">
                        <CornerUpLeft class="w-3 h-3" />
                        Replying to <span class="text-blue-300">{q.display_name || q.sender_id || q.role || 'Anonymous'}</span>
                    </div>
                    <p class="text-[11px] text-slate-400 line-clamp-2 leading-relaxed italic">
                        {q.text || q.content}
                    </p>
                </div>
            </div>
        {/if}

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

        {#if parsedMetadata?.tool_ref_ids && Array.isArray(parsedMetadata.tool_ref_ids)}
            <div class="flex flex-col gap-4 mb-6">
                {#each parsedMetadata.tool_ref_ids as ref}
                    <div class="animate-in fade-in slide-in-from-bottom-2 duration-300">
                        {#if ref.tool?.toLowerCase().includes('reminder') || ref.tool?.toLowerCase().includes('schedule_task')}
                            <ReminderCard ref_id={ref.ref_id} />
                        {:else if ref.tool?.toLowerCase().includes('autonomous') || ref.tool?.toLowerCase().includes('task')}
                            <TaskCard ref_id={ref.ref_id} collapsed={true} />
                        {:else if ref.tool?.toLowerCase().includes('finance') || ref.tool?.toLowerCase().includes('expense') || ref.tool?.toLowerCase().includes('money') || ref.tool?.toLowerCase().includes('manage_finance')}
                            <FinanceCard ref_id={ref.ref_id} />
                        {:else if ref.tool?.toLowerCase().includes('skill') || ref.tool?.toLowerCase().includes('proposal') || ref.tool?.toLowerCase().includes('suggest')}
                            <PluginProposalCard ref_id={ref.ref_id} />
                        {:else}
                            <!-- Technical Fallback for unrecognized tools -->
                            <div class="p-3 bg-slate-900/40 border border-slate-800 rounded-xl flex items-center justify-between gap-3 text-[10px] text-slate-400 font-mono italic shadow-inner">
                                <div class="flex items-center gap-2">
                                    <div class="w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse"></div>
                                    <span>System Ref: <span class="text-white font-bold">{ref.tool}</span></span>
                                </div>
                                <span class="opacity-50 select-all cursor-help" title={ref.ref_id}>ID: {ref.ref_id?.slice(0, 12)}...</span>
                            </div>
                        {/if}
                    </div>
                {/each}
            </div>
        {/if}

        {#if parsedMetadata?.proposal_slug && !parsedMetadata?.tool_ref_ids?.some((r:any) => r.tool === 'suggest_new_skill')}
            <div class="mb-6 animate-in fade-in slide-in-from-bottom-2 duration-300">
                <PluginProposalCard ref_id={parsedMetadata.proposal_slug} />
            </div>
        {/if}

        {@html renderedContent}

        {#if requiredFields.length > 0 && !formSubmitted}
            <div class="mt-4 p-4 bg-slate-900/50 border border-slate-800/80 rounded-xl flex flex-col gap-4 animate-in fade-in slide-in-from-bottom-2 duration-300 max-w-sm">
                <div class="flex items-center gap-2 border-b border-white/5 pb-2 text-[9px] font-black uppercase tracking-widest text-amber-400">
                    <Cpu class="w-3.5 h-3.5 animate-pulse" />
                    <span>Interactive Clarification</span>
                </div>

                {#each requiredFields as field}
                    <div class="flex flex-col gap-1.5">
                        <label class="text-[10px] font-bold text-slate-400 uppercase tracking-wide">{field.label}</label>
                        
                        {#if field.type === 'text'}
                            <input 
                                type="text" 
                                bind:value={formValues[field.name]}
                                placeholder="Type answer here..."
                                class="px-3.5 py-2 rounded-lg bg-slate-950/60 border border-slate-800 text-xs text-white placeholder-slate-600 focus:border-amber-500/50 focus:outline-none transition-all"
                            />
                        {:else if field.type === 'select'}
                            <div class="flex flex-wrap gap-2">
                                {#each field.options as opt}
                                    <button
                                        onclick={() => formValues[field.name] = opt}
                                        class="px-3 py-1.5 rounded-lg border text-[11px] font-bold transition-all duration-300
                                            {formValues[field.name] === opt ? 
                                             'bg-amber-500/10 border-amber-500 text-amber-400' : 
                                             'bg-slate-950 border-slate-800 text-slate-400 hover:border-slate-700 hover:text-slate-200'}"
                                    >
                                        {opt}
                                    </button>
                                {/each}
                            </div>
                        {:else if field.type === 'button' || field.type === 'approval'}
                            <button
                                onclick={() => submitForm(field.name, 'Approved')}
                                class="w-full py-2.5 rounded-lg bg-gradient-to-r from-amber-500 to-amber-600 text-xs font-bold text-slate-950 hover:from-amber-400 hover:to-amber-500 shadow-lg shadow-amber-500/10 active:scale-[0.98] transition-all flex items-center justify-center gap-1.5"
                            >
                                <UserCheck class="w-4 h-4" />
                                {field.label}
                            </button>
                        {/if}
                    </div>
                {/each}

                {#if requiredFields.some(f => f.type !== 'button' && f.type !== 'approval')}
                    <button
                        onclick={() => submitForm()}
                        class="w-full py-2.5 rounded-lg bg-gradient-to-r from-amber-500 to-amber-600 text-xs font-bold text-slate-950 hover:from-amber-400 hover:to-amber-500 shadow-lg shadow-amber-500/10 active:scale-[0.98] transition-all flex items-center justify-center gap-1.5"
                    >
                        <UserCheck class="w-4 h-4" />
                        Submit Answer
                    </button>
                {/if}
            </div>
        {:else if requiredFields.length > 0 && formSubmitted}
            <div class="mt-4 p-3 bg-emerald-500/5 border border-emerald-500/20 rounded-xl flex items-center gap-2 text-emerald-400 text-xs font-semibold">
                <CheckCircle2 class="w-4 h-4 shrink-0" />
                <span>Response submitted successfully!</span>
            </div>
        {/if}
    </div>
</div>


{#if hoverTooltip.visible}
    <div 
        class="mention-tooltip animate-in fade-in zoom-in-95 duration-150"
        style="position: fixed; left: {hoverTooltip.x}px; top: {hoverTooltip.y}px;"
    >
        <div class="flex items-center gap-3">
            <div class="w-9 h-9 rounded-xl overflow-hidden bg-slate-800 border border-slate-700/50 flex-shrink-0 shadow-md">
                <img src={hoverTooltip.avatarUrl} alt={hoverTooltip.displayName} class="w-full h-full object-cover" />
            </div>
            <div class="flex flex-col min-w-0">
                <span class="text-xs font-black text-slate-100 truncate leading-none mb-1">@{hoverTooltip.displayName}</span>
                <span class="text-[9px] font-bold text-purple-400/90 font-mono tracking-wider uppercase leading-none">@{hoverTooltip.externalId}</span>
            </div>
        </div>
    </div>
{/if}

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

    /* Premium Mentions Badge Styling (Violet HSL Gradient) */
    :global(.mention-pill) {
        display: inline-flex !important;
        align-items: center;
        gap: 0.25rem;
        padding: 0.15rem 0.55rem;
        border-radius: 9999px;
        font-size: 0.85em;
        font-weight: 800;
        color: #c084fc !important; /* Soft Purple/Violet */
        background: rgba(192, 132, 252, 0.08) !important;
        border: 1px solid rgba(192, 132, 252, 0.2) !important;
        box-shadow: 0 0 12px rgba(192, 132, 252, 0.05);
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1) !important;
        cursor: pointer;
        user-select: none;
        text-decoration: none !important;
        margin: 0 0.15rem;
    }

    :global(.mention-pill:hover) {
        background: rgba(192, 132, 252, 0.2) !important;
        border-color: rgba(192, 132, 252, 0.5) !important;
        box-shadow: 0 0 18px rgba(192, 132, 252, 0.25);
        transform: translateY(-0.5px);
        color: #e9d5ff !important; /* Lighter Purple */
    }

    /* Premium Mention Tooltip */
    :global(.mention-tooltip) {
        transform: translate(-50%, -100%);
        background: rgba(15, 23, 42, 0.95) !important;
        backdrop-filter: blur(12px) !important;
        border: 1px solid rgba(192, 132, 252, 0.3) !important;
        padding: 0.6rem 0.9rem !important;
        border-radius: 1rem !important;
        box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.6), 0 0 20px rgba(192, 132, 252, 0.15) !important;
        z-index: 99999 !important;
        pointer-events: none !important;
        min-width: 170px;
        margin-top: -6px;
    }

    /* Tooltip Arrow */
    :global(.mention-tooltip::after) {
        content: '';
        position: absolute;
        bottom: 0;
        left: 50%;
        transform: translate(-50%, 100%);
        border-width: 5px;
        border-style: solid;
        border-color: rgba(15, 23, 42, 0.95) transparent transparent transparent;
    }
</style>
