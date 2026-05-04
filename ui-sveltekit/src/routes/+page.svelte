<script lang="ts">
    import {Send, Terminal, Cpu, Bot, User, Sparkles} from 'lucide-svelte';
    import {chatStore} from '$lib/stores/chat.svelte';
    import ToolResult from '$lib/components/ToolResult.svelte';
    import ChatBubble from '$lib/components/ChatBubble.svelte';
    import Dialog from '$lib/components/Dialog.svelte';
    import {onMount} from 'svelte';
    import { popupStore } from '$lib/stores/popup.svelte';

    let inputMessage = $state('');
    let scrollContainer = $state<HTMLElement | null>(null);
    let isDialogOpen = $state(false);

    function openExamplePopup() {
        popupStore.open({
            title: 'Example Popup',
            width: 'max-w-xl',
            contentSnippet: exampleContent,
            footerSnippet: exampleFooter
        });
    }

    function toggleDialog() {
        isDialogOpen = !isDialogOpen;
    }

    function openNestedPopup() {
        popupStore.open({
            title: 'Nested Popup',
            width: 'max-w-md',
            contentSnippet: nestedContent,
            closeOnOutsideClick: true
        });
    }

    onMount(() => {
        chatStore.fetchMessages();
    });

    // Auto-scroll to bottom on new messages
    $effect(() => {
        if (chatStore.messages.length && scrollContainer) {
            scrollContainer.scrollTo({
                top: scrollContainer.scrollHeight,
                behavior: 'smooth'
            });
        }
    });
    async function handleSubmit() {

        if (!inputMessage.trim() || chatStore.loading) return;
        const msg = inputMessage;
        inputMessage = '';
        await chatStore.sendMessage(msg);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSubmit();
        }
    }
</script>

{#snippet exampleContent()}
    <div class="space-y-4">
        <p class="text-zinc-400">This is a reusable popup component inspired by Supabase console.</p>
        <div class="rounded-lg bg-zinc-950 p-4 border border-zinc-800">
            <p class="text-xs font-mono text-emerald-500">// Features included:</p>
            <ul class="mt-2 space-y-1 text-xs font-mono text-zinc-500">
                <li>- Horizontal slide transition</li>
                <li>- Stackable (Z-index management)</li>
                <li>- Global state management</li>
                <li>- Outside click to close</li>
                <li>- Scrollable content area</li>
            </ul>
        </div>
        <button 
            onclick={openNestedPopup}
            class="w-full py-2 bg-zinc-800 hover:bg-zinc-700 rounded text-xs font-medium transition-colors"
        >
            Open Another Popup
        </button>
        <div class="h-[800px] flex items-center justify-center border border-dashed border-zinc-800 rounded mt-4">
            <span class="text-zinc-600 text-xs">Scroll to see fixed header/footer</span>
        </div>
    </div>
{/snippet}

{#snippet exampleFooter()}
    <div class="flex justify-end gap-3">
        <button 
            onclick={() => popupStore.closeLast()}
            class="px-4 py-2 text-xs font-medium text-zinc-400 hover:text-zinc-200"
        >
            Cancel
        </button>
        <button 
            onclick={() => popupStore.closeLast()}
            class="px-4 py-2 text-xs font-medium bg-emerald-600 hover:bg-emerald-500 rounded text-white"
        >
            Save Changes
        </button>
    </div>
{/snippet}

{#snippet nestedContent()}
    <div class="space-y-4">
        <p class="text-zinc-400">This popup is stacked on top of the previous one.</p>
        <div class="p-8 border border-zinc-800 rounded-lg text-center">
            <Sparkles class="w-8 h-8 text-emerald-500 mx-auto mb-2" />
            <p class="text-sm">Multi-layer popup management works!</p>
        </div>
    </div>
{/snippet}

{#snippet dialogFooter()}
    <div class="flex justify-end gap-3 w-full">
        <button 
            onclick={toggleDialog}
            class="px-4 py-2 text-xs font-medium text-zinc-400 hover:text-zinc-200"
        >
            Cancel
        </button>
        <button 
            onclick={toggleDialog}
            class="px-4 py-2 text-xs font-medium bg-zinc-100 hover:bg-zinc-200 rounded text-zinc-950 transition-colors"
        >
            Confirm Action
        </button>
    </div>
{/snippet}

<Dialog 
    isOpen={isDialogOpen} 
    onClose={toggleDialog} 
    title="Reusable Dialog" 
    clickOutside={true}
    footer={dialogFooter}
>
    <div class="space-y-4">
        <p class="text-zinc-400 text-sm">
            This is the new reusable Dialog component. It supports snippets for content and footer, 
            has a dim overlay, and handles overflow automatically.
        </p>
        <div class="grid grid-cols-2 gap-4">
            <div class="p-4 rounded-lg bg-zinc-950 border border-zinc-800">
                <p class="text-[10px] font-bold text-zinc-500 uppercase mb-2">Features</p>
                <ul class="text-xs space-y-1 text-zinc-400">
                    <li>• Click outside to close</li>
                    <li>• ESC key support</li>
                    <li>• Scale transition</li>
                    <li>• Scrollable content</li>
                </ul>
            </div>
            <div class="p-4 rounded-lg bg-zinc-950 border border-zinc-800">
                <p class="text-[10px] font-bold text-zinc-500 uppercase mb-2">Customization</p>
                <ul class="text-xs space-y-1 text-zinc-400">
                    <li>• Title optional</li>
                    <li>• Custom max-width</li>
                    <li>• Custom max-height</li>
                    <li>• Render snippets</li>
                </ul>
            </div>
        </div>
        <div class="h-64 flex flex-col items-center justify-center border border-dashed border-zinc-800 rounded-lg bg-zinc-900/30">
            <span class="text-zinc-600 text-xs">Scrollable area demo</span>
            <div class="mt-4 w-1/2 h-1 bg-zinc-800 rounded-full overflow-hidden">
                <div class="h-full bg-emerald-500 w-1/3"></div>
            </div>
        </div>
        <p class="text-zinc-500 text-xs leading-relaxed">
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. 
            Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
        </p>
    </div>
</Dialog>

<!-- Header -->
<header class="h-14 border-b border-zinc-800 flex justify-between items-center px-6 bg-[#09090b]/80 backdrop-blur-md sticky top-0 z-20">
            <div class="flex items-center gap-4">
                <div class="flex items-center gap-2 text-zinc-400">
                    <span class="text-[10px] font-bold uppercase tracking-widest">Workspace</span>
                    <span class="text-zinc-700">/</span>
                    <span class="text-xs font-medium text-zinc-200">Arta Orchestrator</span>
                </div>
            </div>
            <div class="flex items-center gap-3">
                <button 
                    onclick={toggleDialog}
                    class="px-3 py-1.5 rounded-lg bg-emerald-600/20 hover:bg-emerald-600/30 border border-emerald-600/30 text-[10px] font-bold uppercase tracking-wider transition-colors text-emerald-400"
                >
                    Test Dialog
                </button>
                <button 
                    onclick={openExamplePopup}
                    class="px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-[10px] font-bold uppercase tracking-wider transition-colors"
                >
                    Open Popup
                </button>
                <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20">
                    <div class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></div>
                    <span class="text-[10px] text-emerald-500 font-bold uppercase tracking-wider">Gateway Active</span>
                </div>
                <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-900 border border-zinc-800">
                    <Cpu class="w-3 h-3 text-zinc-500"/>
                    <span class="text-[10px] text-zinc-400 font-bold uppercase tracking-wider">Llama 3.1 70B</span>
                </div>
            </div>
        </header>

        <!-- Messages -->
        <main bind:this={scrollContainer} class="flex-1 overflow-y-auto px-6 py-8 space-y-10 scroll-smooth">
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
                            <span class="text-[10px] text-zinc-600 font-mono">12:45 PM</span>
                            </div>

                            {#if msg.toolCalls && msg.toolCalls.length > 0}
                            <div class="space-y-3">
                                {#each msg.toolCalls as tc}
                                    <ToolResult args="" tool={tc.tool} result={tc.result} />
                                {/each}
                            </div>
                            {/if}

                            <ChatBubble content={msg.content} thought={msg.thought} />                        </div>
                    </div>
                {/each}

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
                <div class="relative bg-zinc-900/50 border border-zinc-800 rounded-2xl p-1 shadow-2xl focus-within:border-zinc-700 transition-colors">
                    <textarea
                            bind:value={inputMessage}
                            onkeydown={handleKeydown}
                            placeholder="Message Arta..."
                            class="w-full bg-transparent border-none focus:ring-0 text-sm py-4 px-5 min-h-[60px] max-h-48 resize-none placeholder:text-zinc-600 text-zinc-200"
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
                <p class="text-[9px] text-zinc-700 text-center mt-4 uppercase tracking-[0.2em] font-bold">
                    Experimental AI System — Powered by Axum & SvelteKit
                </p>
            </div>
        </footer>
    <!-- </div> (Removed) -->
<!-- </div> (Removed) -->

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
