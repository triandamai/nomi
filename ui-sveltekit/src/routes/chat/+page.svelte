<script lang="ts">
    import {
        Send,
        Bot,
        User,
        Sparkles,
        MessageSquarePlus,
        Share2,
        Paperclip,
        X,
        Image as ImageIcon,
        FileAudio,
        FileText,
        Loader2,
        Wrench
    } from 'lucide-svelte';
    import {chatStore} from '$lib/stores/chat.svelte';
    import {conversationStore} from '$lib/stores/conversation.svelte';
    import {chatApi} from '$lib/api/client';
    import {formatTokenCount} from '$lib/utils';
    import ToolResult from '$lib/components/ToolResult.svelte';
    import ChatBubble from '$lib/components/ChatBubble.svelte';
    import PricingPopUp from '$lib/components/PricingPopUp.svelte';
    import {onMount, tick} from 'svelte';
    import {eventBus} from "$lib/utils";
    import {goto} from '$app/navigation';
    import {popupStore} from '$lib/stores/popup.svelte';

    let inputMessage = $state('');
    let scrollContainer = $state<HTMLElement | null>(null);
    let isNearBottom = true;
    let fileInput = $state<HTMLInputElement | null>(null);
    let selectedFile = $state<File | null>(null);
    let isUploading = $state(false);

    function handleScroll() {
        if (!scrollContainer) return;
        const threshold = 150; // pixels from bottom to be considered "near bottom"
        const position = scrollContainer.scrollHeight - scrollContainer.scrollTop - scrollContainer.clientHeight;
        isNearBottom = position < threshold;
    }

    onMount(() => {
        chatStore.fetchMessages();
    });

    function scrollToBottom() {
        tick().then(() => {
            if (scrollContainer) {
                scrollContainer.scrollTo({
                    top: scrollContainer.scrollHeight,
                    behavior: 'smooth'
                });
            }
        });
    }

    function handleToggleThought(isExpanded: boolean, isLast: boolean) {
        if (isExpanded && isLast) {
            scrollToBottom();
        }
    }

    // Auto-scroll to bottom on new messages, thoughts, or typing
    $effect(() => {
        // Track dependencies
        chatStore.messages.length;
        chatStore.currentThought;
        chatStore.isTyping;
        chatStore.activeTool;
        isUploading
        isNearBottom


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
        if ((!inputMessage.trim() && !selectedFile) || chatStore.loading || !conversationStore.activeConversationId) return;

        let media = undefined;
        if (selectedFile) {
            isUploading = true;
            try {
                const res = await chatApi.uploadFile(selectedFile);
                const fileUrl = res.data; // This is the unique_name from backend

                const type = selectedFile.type;
                if (type.startsWith('image/')) media = {image_url: fileUrl};
                else if (type.startsWith('audio/')) media = {audio_url: fileUrl};
                else if (type.startsWith('video/')) media = {video_url: fileUrl};
                else media = {doc_url: fileUrl};
            } catch (err) {
                console.error("Upload failed", err);
                return;
            } finally {
                isUploading = false;
            }
        }

        const msg = inputMessage;
        inputMessage = '';
        selectedFile = null;

        // Force scroll to bottom when user sends a message
        isNearBottom = true;


        const response = await chatStore.sendMessage(msg, media);
        if (!response.isSuccess && response.isLimit) {
            popupStore.open({
                title: 'Memory Full',
                width: 'max-w-xl',
                contentSnippet: tokenLimitSnippet
            });
        }
    }

    function handleFileSelect(e: Event) {
        const target = e.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            selectedFile = target.files[0];
            scrollToBottom();
        }
    }

    function removeFile() {
        selectedFile = null;
        if (fileInput) fileInput.value = '';
        scrollToBottom();
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSubmit();
        }
    }

    function startFirstConversation() {
        conversationStore.addConversation("My First Soul").then(conv => {
            if (conv) conversationStore.setActive(conv.id);
        });
    }

    onMount(() => {
        const unsubscribe = eventBus.subscribe("token-limit-reached", (message) => {
            popupStore.open({
                title: 'Memory Full',
                width: 'max-w-xl',
                contentSnippet: tokenLimitSnippet
            });
        });

        eventBus.subscribe("", () => {

        })
        eventBus.emit("load", {})

        return () => unsubscribe();
    })
</script>

{#snippet tokenLimitSnippet()}
    <PricingPopUp/>
{/snippet}

<!-- Messages -->
<main
        bind:this={scrollContainer}
        onscroll={handleScroll}
        class="flex-1 overflow-y-auto px-6 pt-8 {selectedFile ? 'pb-64' : 'pb-48'} space-y-10 scroll-smooth"
>
    {#if conversationStore.conversations.length === 0}
        <div class="h-full flex flex-col items-center justify-center max-w-lg mx-auto text-center space-y-8 animate-in fade-in zoom-in duration-700">
            <div class="relative">
                <div class="absolute -inset-8 bg-blue-500/20 blur-3xl rounded-full"></div>
                <div class="w-20 h-20 bg-blue-600 rounded-[28px] flex items-center justify-center relative shadow-2xl shadow-blue-500/20">
                    <Sparkles class="w-10 h-10 text-white fill-white"/>
                </div>
            </div>

            <div class="space-y-3">
                <h2 class="text-3xl font-black text-white tracking-tight">Your Life, Decoded</h2>
                <p class="text-sm text-slate-400 leading-relaxed max-w-sm mx-auto">
                    Nomi is ready to help you orchestrate your multimodal life infrastructure. Start a conversation to
                    begin.
                </p>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 w-full">
                <button
                        onclick={startFirstConversation}
                        class="group p-6 bg-slate-900/50 hover:bg-blue-600/10 border border-slate-800 hover:border-blue-500/50 rounded-2xl text-left transition-all"
                >
                    <MessageSquarePlus class="w-6 h-6 text-slate-500 group-hover:text-blue-400 mb-4 transition-colors"/>
                    <h3 class="text-sm font-bold text-slate-200 group-hover:text-blue-300 transition-colors">Start a new
                        Soul</h3>
                    <p class="text-[11px] text-slate-500 group-hover:text-blue-400/70 mt-1 transition-colors leading-snug">
                        Create a new intelligent sandbox.</p>
                </button>

                <button
                        onclick={() => goto('/rag')}
                        class="group p-6 bg-slate-900/50 hover:bg-emerald-600/10 border border-slate-800 hover:border-emerald-500/50 rounded-2xl text-left transition-all"
                >
                    <Share2 class="w-6 h-6 text-slate-500 group-hover:text-emerald-400 mb-4 transition-colors"/>
                    <h3 class="text-sm font-bold text-slate-200 group-hover:text-emerald-300 transition-colors">
                        Knowledge Base</h3>
                    <p class="text-[11px] text-slate-500 group-hover:text-emerald-400/70 mt-1 transition-colors leading-snug">
                        Upload documents to sharpen Nomi's memory.</p>
                </button>
            </div>
        </div>
    {:else if chatStore.messages.length === 0 && !chatStore.loading}
        <div class="h-full flex flex-col items-center justify-center max-w-lg mx-auto text-center space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-700">
            <div class="w-16 h-16 bg-slate-900 border border-slate-800 rounded-2xl flex items-center justify-center shadow-2xl">
                <MessageSquarePlus class="w-8 h-8 text-slate-500"/>
            </div>
            <div class="space-y-2">
                <h3 class="text-xl font-bold text-slate-200">New Conversation</h3>
                <p class="text-sm text-slate-500 leading-relaxed">
                    This conversation is empty. Send a message to start interacting with your agent.
                </p>
            </div>
            <div class="flex flex-wrap justify-center gap-2 max-w-sm">
                <button
                        onclick={() => inputMessage = "Hello! How can you help me today?"}
                        class="px-3 py-1.5 bg-slate-900 hover:bg-slate-800 border border-slate-800 rounded-lg text-[11px] text-slate-400 hover:text-slate-200 transition-all"
                >
                    "How can you help me?"
                </button>
                <button
                        onclick={() => inputMessage = "What are your capabilities?"}
                        class="px-3 py-1.5 bg-slate-900 hover:bg-slate-800 border border-slate-800 rounded-lg text-[11px] text-slate-400 hover:text-slate-200 transition-all"
                >
                    "What are your capabilities?"
                </button>
            </div>
        </div>
    {:else}
        <div class="max-w-4xl mx-auto space-y-10">
            {#if chatStore.hasMore}
                <div class="flex justify-center">
                    <button
                            onclick={() => chatStore.fetchMessages(true)}
                            disabled={chatStore.loading}
                            class="text-[10px] font-bold uppercase tracking-widest text-slate-500 hover:text-slate-300 transition-colors disabled:opacity-50"
                    >
                        {chatStore.loading ? 'Loading...' : 'Load Previous Messages'}
                    </button>
                </div>
            {/if}

            {#each chatStore.messages as msg, index (msg.id)}
                <div class="group flex gap-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
                    <div class="flex-shrink-0 pt-1">
                        {#if msg.role === 'user'}
                            <div class="w-8 h-8 rounded-lg bg-slate-100 flex items-center justify-center">
                                <User class="w-4 h-4 text-slate-950"/>
                            </div>
                        {:else}
                            <div class="w-8 h-8 rounded-lg bg-slate-900 border border-slate-800 flex items-center justify-center">
                                <Bot class="w-4 h-4 text-slate-400"/>
                            </div>
                        {/if}
                    </div>

                    <div class="flex-1 flex flex-col min-w-0 space-y-4">
                        <div class="flex items-center gap-2">
                        <span class="text-xs font-bold uppercase tracking-wider {msg.role === 'user' ? 'text-slate-300' : 'text-blue-400'}">
                            {msg.role === 'user' ? 'Human' : 'Nomi'}
                            {#if msg.role !== 'user'}<span
                                    class="font-mono ml-2 text-[10px] text-slate-500">- {formatTokenCount(msg.total_tokens)}
                                Token</span>{/if}
                        </span>
                        </div>

                        {#if msg.toolCalls && msg.toolCalls.length > 0}
                            <div class="space-y-3">
                                {#each msg.toolCalls as tc}
                                    <ToolResult args="" tool={tc.tool} result={tc.result}/>
                                {/each}
                            </div>
                        {/if}

                        <ChatBubble
                                content={msg.content}
                                thought={msg.thought}
                                image_url={msg.image_url}
                                onToggleThought={(expanded: boolean) => handleToggleThought(expanded, index === chatStore.messages.length - 1)}
                        />
                    </div>
                </div>
            {/each}

            {#if chatStore.currentThought || chatStore.isTyping}
                <div class="group flex gap-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
                    <div class="flex-shrink-0 pt-1">
                        <div class="w-8 h-8 rounded-lg bg-slate-900 border border-slate-800 flex items-center justify-center">
                            <Bot class="w-4 h-4 text-slate-400"/>
                        </div>
                    </div>

                    <div class="flex-1 flex flex-col min-w-0 space-y-4">
                        <div class="flex items-center gap-2">
                            <span class="text-xs font-bold uppercase tracking-wider text-blue-400">Nomi</span>
                            {#if chatStore.isTyping}
                                <div class="flex gap-1 ml-2">
                                    <div class="w-1 h-1 bg-blue-500 rounded-full animate-bounce"></div>
                                    <div class="w-1 h-1 bg-blue-500 rounded-full animate-bounce [animation-delay:0.2s]"></div>
                                    <div class="w-1 h-1 bg-blue-500 rounded-full animate-bounce [animation-delay:0.4s]"></div>
                                </div>
                            {/if}
                            {#if chatStore.activeTool}
                                <div class="flex items-center gap-1.5 ml-3 px-2 py-0.5 bg-blue-500/10 border border-blue-500/20 rounded-full animate-in fade-in zoom-in duration-300">
                                    <Wrench class="w-2.5 h-2.5 text-blue-500 animate-pulse"/>
                                    <span class="text-[9px] font-black uppercase tracking-widest text-blue-500/90">Using {chatStore.activeTool.replace(/_/g, ' ')}</span>
                                </div>
                            {/if}
                        </div>

                        {#if chatStore.currentThought}
                            <div class="p-4 rounded-2xl bg-slate-900/30 border border-slate-800/50">
                                <div class="flex items-center gap-2 mb-2">
                                    <Sparkles class="w-3 h-3 text-blue-500"/>
                                    <span class="text-[10px] font-bold uppercase tracking-widest text-blue-400/70">Thinking...</span>
                                </div>
                                <p class="text-xs text-slate-400 italic leading-relaxed">
                                    {chatStore.currentThought}
                                </p>
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}

            {#if chatStore.loading}
                <div class="flex gap-6 animate-pulse opacity-50">
                    <div class="w-8 h-8 rounded-lg bg-slate-900 border border-slate-800"></div>
                    <div class="flex-1 space-y-4 pt-1">
                        <div class="h-2.5 w-24 bg-slate-800 rounded"></div>
                        <div class="space-y-2">
                            <div class="h-2 w-full bg-slate-800 rounded"></div>
                            <div class="h-2 w-[90%] bg-slate-800 rounded"></div>
                        </div>
                    </div>
                </div>
            {/if}

            {#if chatStore.error}
                <div class="p-4 rounded-xl bg-rose-500/5 border border-rose-500/10 flex items-center gap-3">
                    <div class="w-2 h-2 rounded-full bg-rose-500"></div>
                    <p class="text-xs font-medium text-rose-400">{chatStore.error}</p>
                </div>
            {/if}
        </div>
    {/if}
</main>

<!-- Input -->
<footer class="absolute bottom-0 left-0 right-0 p-4 md:p-8 pointer-events-none z-20">
    <div class="max-w-4xl mx-auto pointer-events-auto">
        <div class="relative transition-all duration-500 rounded-2xl {chatStore.isTyping ? 'ring-2 ring-blue-500/20 shadow-[0_0_30px_-5px_rgba(59,130,246,0.3)]' : ''}">
            <div class="relative bg-[#0f172a]/80 border border-slate-800 rounded-2xl p-1 shadow-2xl focus-within:border-slate-700 transition-colors backdrop-blur-xl">

                {#if selectedFile}
                    <div class="px-4 md:px-5 pt-4">
                        <div class="inline-flex items-center gap-3 p-2 bg-slate-800/50 border border-slate-700 rounded-xl animate-in fade-in slide-in-from-left-2">
                            <div class="w-8 h-8 md:w-10 md:h-10 bg-slate-950 rounded-lg flex items-center justify-center border border-slate-700">
                                {#if selectedFile.type.startsWith('image/')}
                                    <ImageIcon class="w-4 h-4 md:w-5 md:h-5 text-blue-500"/>
                                {:else if selectedFile.type.startsWith('audio/')}
                                    <FileAudio class="w-4 h-4 md:w-5 md:h-5 text-emerald-500"/>
                                {:else}
                                    <FileText class="w-4 h-4 md:w-5 md:h-5 text-slate-400"/>
                                {/if}
                            </div>
                            <div class="flex flex-col min-w-0">
                                <span class="text-[10px] md:text-[11px] font-bold text-slate-200 truncate max-w-[100px] md:max-w-[150px]">{selectedFile.name}</span>
                                <span class="text-[8px] md:text-[9px] text-slate-500 uppercase">{(selectedFile.size / 1024 / 1024).toFixed(2)}
                                    MB</span>
                            </div>
                            <button
                                    onclick={removeFile}
                                    class="p-1 hover:bg-slate-700 rounded-lg transition-colors ml-1"
                            >
                                <X class="w-3.5 h-3.5 md:w-4 md:h-4 text-slate-500"/>
                            </button>
                        </div>
                    </div>
                {/if}

                <textarea
                        bind:value={inputMessage}
                        onkeydown={handleKeydown}
                        placeholder="Message Nomi..."
                        class="w-full bg-transparent border-none focus:ring-0 text-sm py-3 md:py-4 px-4 md:px-5 min-h-[50px] md:min-h-[60px] max-h-32 md:max-h-48 resize-none placeholder:text-slate-600 text-slate-200 focus-within:border-slate-700 focus:border-0 focus:outline-0"
                ></textarea>

                <div class="flex items-center justify-between px-2 md:px-3 pb-2 pt-1">
                    <div class="flex items-center gap-0.5 md:gap-1">
                        <input
                                type="file"
                                bind:this={fileInput}
                                onchange={handleFileSelect}
                                class="hidden"
                                accept="image/*,audio/*,video/*,.pdf,.doc,.docx,.txt"
                        />
                        <button
                                onclick={() => fileInput?.click()}
                                class="p-2 text-slate-500 hover:text-slate-300 transition-colors"
                        >
                            <Paperclip class="w-4 h-4"/>
                        </button>
                        <button class="p-2 text-slate-500 hover:text-slate-300 transition-colors">
                            <Sparkles class="w-4 h-4"/>
                        </button>
                    </div>

                    <button
                            onclick={handleSubmit}
                            disabled={(!inputMessage.trim() && !selectedFile) || chatStore.loading || isUploading}
                            class="flex items-center gap-2 px-4 py-2 rounded-xl bg-blue-600 text-white text-xs font-bold hover:bg-blue-500 disabled:opacity-20 transition-all shadow-lg shadow-blue-500/20"
                    >
                        {#if isUploading}
                            <Loader2 class="w-3.5 h-3.5 animate-spin"/>
                            <span class="hidden sm:inline">Uploading...</span>
                        {:else}
                            <span class="hidden sm:inline">Send</span>
                            <Send class="w-3.5 h-3.5"/>
                        {/if}
                    </button>
                </div>
            </div>
        </div>
    </div>
</footer>

<style>
    :global(body) {
        background-color: #0f172a;
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
        background: #1e293b;
        border-radius: 10px;
    }

    ::-webkit-scrollbar-thumb:hover {
        background: #334155;
    }
</style>
