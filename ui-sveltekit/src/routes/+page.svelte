<script lang="ts">
    import {Send, Bot, User, Sparkles} from 'lucide-svelte';
    import {chatStore} from '$lib/stores/chat.svelte';
    import ToolResult from '$lib/components/ToolResult.svelte';
    import ChatBubble from '$lib/components/ChatBubble.svelte';
    import {onMount, tick} from 'svelte';

    let inputMessage = $state('');
    let scrollContainer = $state<HTMLElement | null>(null);
    let isNearBottom = true;

    function handleScroll() {
        if (!scrollContainer) return;
        const threshold = 150; // pixels from bottom to be considered "near bottom"
        const position = scrollContainer.scrollHeight - scrollContainer.scrollTop - scrollContainer.clientHeight;
        isNearBottom = position < threshold;
    }

    onMount(() => {
        chatStore.fetchMessages();
    });

    // Auto-scroll to bottom on new messages, thoughts, or typing
    $effect(() => {
        // Track dependencies
        chatStore.messages.length;
        chatStore.currentThought;
        chatStore.isTyping;
        
        tick().then(() => {
            if (scrollContainer && isNearBottom) {
                scrollContainer.scrollTo({
                    top: scrollContainer.scrollHeight,
                    behavior: 'auto'
                });
            }
        });
    });

    async function handleSubmit() {
        if (!inputMessage.trim() || chatStore.loading) return;
        const msg = inputMessage;
        inputMessage = '';
        
        // Force scroll to bottom when user sends a message
        isNearBottom = true;
        
        await chatStore.sendMessage(msg);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSubmit();
        }
    }
</script>

<!-- Messages -->
<main 
    bind:this={scrollContainer} 
    onscroll={handleScroll}
    class="flex-1 overflow-y-auto px-6 py-8 space-y-10 scroll-smooth"
>
    <div class="max-w-4xl mx-auto space-y-10">
        {#if chatStore.hasMore}
            <div class="flex justify-center">
                <button 
                    onclick={() => chatStore.fetchMessages(true)}
                    disabled={chatStore.loading}
                    class="text-[10px] font-bold uppercase tracking-widest text-zinc-500 hover:text-zinc-300 transition-colors disabled:opacity-50"
                >
                    {chatStore.loading ? 'Loading...' : 'Load Previous Messages'}
                </button>
            </div>
        {/if}

        {#each chatStore.messages as msg (msg.id)}
            <div class="group flex gap-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
                <div class="flex-shrink-0 pt-1">
                    {#if msg.role === 'user'}
                        <div class="w-8 h-8 rounded-lg bg-zinc-100 flex items-center justify-center">
                            <User class="w-4 h-4 text-zinc-950"/>
                        </div>
                    {:else}
                        <div class="w-8 h-8 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-center">
                            <Bot class="w-4 h-4 text-zinc-400"/>
                        </div>
                    {/if}
                </div>

                <div class="flex-1 flex flex-col min-w-0 space-y-4">
                    <div class="flex items-center gap-2">
                    <span class="text-xs font-bold uppercase tracking-wider text-zinc-400">
                        {msg.role === 'user' ? 'Human' : 'Arta AI'}
                    </span>
                    </div>

                    {#if msg.toolCalls && msg.toolCalls.length > 0}
                    <div class="space-y-3">
                        {#each msg.toolCalls as tc}
                            <ToolResult args="" tool={tc.tool} result={tc.result} />
                        {/each}
                    </div>
                    {/if}

                    <ChatBubble content={msg.content} thought={msg.thought} />
                </div>
            </div>
        {/each}

        {#if chatStore.currentThought || chatStore.isTyping}
            <div class="group flex gap-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
                <div class="flex-shrink-0 pt-1">
                    <div class="w-8 h-8 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-center">
                        <Bot class="w-4 h-4 text-zinc-400"/>
                    </div>
                </div>

                <div class="flex-1 flex flex-col min-w-0 space-y-4">
                    <div class="flex items-center gap-2">
                        <span class="text-xs font-bold uppercase tracking-wider text-zinc-400">Arta AI</span>
                        {#if chatStore.isTyping}
                            <div class="flex gap-1 ml-2">
                                <div class="w-1 h-1 bg-zinc-500 rounded-full animate-bounce"></div>
                                <div class="w-1 h-1 bg-zinc-500 rounded-full animate-bounce [animation-delay:0.2s]"></div>
                                <div class="w-1 h-1 bg-zinc-500 rounded-full animate-bounce [animation-delay:0.4s]"></div>
                            </div>
                        {/if}
                    </div>

                    {#if chatStore.currentThought}
                        <div class="p-4 rounded-2xl bg-zinc-900/30 border border-zinc-800/50">
                            <div class="flex items-center gap-2 mb-2">
                                <Sparkles class="w-3 h-3 text-zinc-500" />
                                <span class="text-[10px] font-bold uppercase tracking-widest text-zinc-500">Thinking...</span>
                            </div>
                            <p class="text-xs text-zinc-400 italic leading-relaxed">
                                {chatStore.currentThought}
                            </p>
                        </div>
                    {/if}
                </div>
            </div>
        {/if}

        {#if chatStore.loading}
            <div class="flex gap-6 animate-pulse opacity-50">
                <div class="w-8 h-8 rounded-lg bg-zinc-900 border border-zinc-800"></div>
                <div class="flex-1 space-y-4 pt-1">
                    <div class="h-2.5 w-24 bg-zinc-800 rounded"></div>
                    <div class="space-y-2">
                        <div class="h-2 w-full bg-zinc-800 rounded"></div>
                        <div class="h-2 w-[90%] bg-zinc-800 rounded"></div>
                    </div>
                </div>
            </div>
        {/if}

        {#if chatStore.error}
            <div class="p-4 rounded-xl bg-red-500/5 border border-red-500/10 flex items-center gap-3">
                <div class="w-2 h-2 rounded-full bg-red-500"></div>
                <p class="text-xs font-medium text-red-400">{chatStore.error}</p>
            </div>
        {/if}
    </div>
</main>

<!-- Input -->
<footer class="p-8 bg-gradient-to-t from-[#09090b] via-[#09090b] to-transparent">
    <div class="max-w-4xl mx-auto">
        <div class="relative transition-all duration-500 rounded-2xl {chatStore.isTyping ? 'ring-2 ring-emerald-500/20 shadow-[0_0_30px_-5px_rgba(16,185,129,0.3)]' : ''}">
            <div class="relative bg-zinc-900/50 border border-zinc-800 rounded-2xl p-1 shadow-2xl focus-within:border-zinc-700 transition-colors">
                <textarea
                        bind:value={inputMessage}
                        onkeydown={handleKeydown}
                        placeholder="Message Arta..."
                        class="w-full bg-transparent border-none focus:ring-0 text-sm py-4 px-5 min-h-[60px] max-h-48 resize-none placeholder:text-zinc-600 text-zinc-200 focus-within:border-zinc-700 focus:border-0 focus:outline-0"
                ></textarea>

                <div class="flex items-center justify-between px-3 pb-2 pt-1">
                    <div class="flex items-center gap-1">
                        <button class="p-2 text-zinc-500 hover:text-zinc-300 transition-colors">
                            <Sparkles class="w-4 h-4"/>
                        </button>
                    </div>

                    <button
                            onclick={handleSubmit}
                            disabled={!inputMessage.trim() || chatStore.loading}
                            class="flex items-center gap-2 px-4 py-2 rounded-xl bg-zinc-100 text-zinc-950 text-xs font-bold hover:bg-zinc-200 disabled:opacity-20 transition-all shadow-lg"
                    >
                        Send
                        <Send class="w-3.5 h-3.5"/>
                    </button>
                </div>
            </div>
        </div>
        <p class="text-[9px] text-zinc-700 text-center mt-4 uppercase tracking-[0.2em] font-bold">
            Experimental AI System — Powered by Axum & SvelteKit
        </p>
    </div>
</footer>

<style>
    :global(body) {
        background-color: #09090b;
        margin: 0;
        height: 100vh;
        width: 100vw;
        overflow: hidden;
    }

    ::-webkit-scrollbar {
        width: 6px;
    }

    ::-webkit-scrollbar-track {
        background: transparent;
    }

    ::-webkit-scrollbar-thumb {
        background: #18181b;
        border-radius: 10px;
    }

    ::-webkit-scrollbar-thumb:hover {
        background: #27272a;
    }
</style>
