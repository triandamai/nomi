<script lang="ts">
    import {Send, Bot, User, Sparkles, MessageSquarePlus, Share2, Paperclip, X, Image as ImageIcon, FileAudio, FileText, Loader2} from 'lucide-svelte';
    import {chatStore} from '$lib/stores/chat.svelte';
    import {conversationStore} from '$lib/stores/conversation.svelte';
    import {chatApi} from '$lib/api/client';
    import ToolResult from '$lib/components/ToolResult.svelte';
    import ChatBubble from '$lib/components/ChatBubble.svelte';
    import {onMount, tick} from 'svelte';
    import {eventBus} from "$lib/utils";
    import {goto} from '$app/navigation';

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
        if ((!inputMessage.trim() && !selectedFile) || chatStore.loading || !conversationStore.activeConversationId) return;
        
        let media = undefined;
        if (selectedFile) {
            isUploading = true;
            try {
                const res = await chatApi.uploadFile(selectedFile);
                const fileUrl = res.data; // This is the unique_name from backend
                
                const type = selectedFile.type;
                if (type.startsWith('image/')) media = { image_url: fileUrl };
                else if (type.startsWith('audio/')) media = { audio_url: fileUrl };
                else if (type.startsWith('video/')) media = { video_url: fileUrl };
                else media = { doc_url: fileUrl };
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
        
        await chatStore.sendMessage(msg, media);
    }

    function handleFileSelect(e: Event) {
        const target = e.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            selectedFile = target.files[0];
        }
    }

    function removeFile() {
        selectedFile = null;
        if (fileInput) fileInput.value = '';
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

    onMount(()=>{
        eventBus.emit("load",{})
    })
</script>

<!-- Messages -->
<main 
    bind:this={scrollContainer} 
    onscroll={handleScroll}
    class="flex-1 overflow-y-auto px-6 py-8 space-y-10 scroll-smooth"
>
    {#if conversationStore.conversations.length === 0}
        <div class="h-full flex flex-col items-center justify-center max-w-lg mx-auto text-center space-y-8 animate-in fade-in zoom-in duration-700">
            <div class="relative">
                <div class="absolute -inset-4 bg-emerald-500/10 blur-3xl rounded-full"></div>
                <div class="w-20 h-20 bg-zinc-900 border border-zinc-800 rounded-[28px] flex items-center justify-center relative">
                    <Sparkles class="w-10 h-10 text-emerald-500" />
                </div>
            </div>
            
            <div class="space-y-3">
                <h2 class="text-2xl font-black text-zinc-100 tracking-tight">Your Agentic Workspace</h2>
                <p class="text-sm text-zinc-400 leading-relaxed">
                    Arta is ready to help you orchestrate your AI agents. Start a conversation to begin your journey.
                </p>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 w-full">
                <button 
                    onclick={startFirstConversation}
                    class="group p-6 bg-zinc-900/50 hover:bg-emerald-900/20 border border-zinc-800 hover:border-emerald-500/50 rounded-2xl text-left transition-all"
                >
                    <MessageSquarePlus class="w-6 h-6 text-zinc-500 group-hover:text-emerald-400 mb-4 transition-colors" />
                    <h3 class="text-sm font-bold text-zinc-200 group-hover:text-emerald-300 transition-colors">Start your first Soul</h3>
                    <p class="text-[11px] text-zinc-500 group-hover:text-emerald-400/70 mt-1 transition-colors leading-snug">Create a new sandbox for your AI interactions.</p>
                </button>

                <button 
                    onclick={() => goto('/rag')}
                    class="group p-6 bg-zinc-900/50 hover:bg-blue-900/20 border border-zinc-800 hover:border-blue-500/50 rounded-2xl text-left transition-all"
                >
                    <Share2 class="w-6 h-6 text-zinc-500 group-hover:text-blue-400 mb-4 transition-colors" />
                    <h3 class="text-sm font-bold text-zinc-200 group-hover:text-blue-300 transition-colors">Knowledge Base</h3>
                    <p class="text-[11px] text-zinc-500 group-hover:text-blue-400/70 mt-1 transition-colors leading-snug">Upload documents to make Arta even smarter.</p>
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
                            {msg.role === 'user' ? 'Human' : 'Nomi'} - {Number(msg.total_tokens) > 0 ? `${msg.total_tokens} Token`:`0`}
                        </span>
                        </div>

                        {#if msg.toolCalls && msg.toolCalls.length > 0}
                        <div class="space-y-3">
                            {#each msg.toolCalls as tc}
                                <ToolResult args="" tool={tc.tool} result={tc.result} />
                            {/each}
                        </div>
                        {/if}

                        <ChatBubble content={msg.content} thought={msg.thought} image_url={msg.image_url} />
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
                            <span class="text-xs font-bold uppercase tracking-wider text-zinc-400">Nomi</span>
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
    {/if}
</main>

<!-- Input -->
<footer class="p-8 bg-gradient-to-t from-[#09090b] via-[#09090b] to-transparent">
    <div class="max-w-4xl mx-auto">
        <div class="relative transition-all duration-500 rounded-2xl {chatStore.isTyping ? 'ring-2 ring-emerald-500/20 shadow-[0_0_30px_-5px_rgba(16,185,129,0.3)]' : ''}">
            <div class="relative bg-zinc-900/50 border border-zinc-800 rounded-2xl p-1 shadow-2xl focus-within:border-zinc-700 transition-colors">
                
                {#if selectedFile}
                    <div class="px-5 pt-4">
                        <div class="inline-flex items-center gap-3 p-2 bg-zinc-800/50 border border-zinc-700 rounded-xl animate-in fade-in slide-in-from-left-2">
                            <div class="w-10 h-10 bg-zinc-900 rounded-lg flex items-center justify-center border border-zinc-700">
                                {#if selectedFile.type.startsWith('image/')}
                                    <ImageIcon class="w-5 h-5 text-emerald-500" />
                                {:else if selectedFile.type.startsWith('audio/')}
                                    <FileAudio class="w-5 h-5 text-blue-500" />
                                {:else}
                                    <FileText class="w-5 h-5 text-zinc-400" />
                                {/if}
                            </div>
                            <div class="flex flex-col">
                                <span class="text-[11px] font-bold text-zinc-200 truncate max-w-[150px]">{selectedFile.name}</span>
                                <span class="text-[9px] text-zinc-500 uppercase">{(selectedFile.size / 1024 / 1024).toFixed(2)} MB</span>
                            </div>
                            <button 
                                onclick={removeFile}
                                class="p-1 hover:bg-zinc-700 rounded-lg transition-colors ml-1"
                            >
                                <X class="w-4 h-4 text-zinc-500" />
                            </button>
                        </div>
                    </div>
                {/if}

                <textarea
                        bind:value={inputMessage}
                        onkeydown={handleKeydown}
                        placeholder="Message Arta..."
                        class="w-full bg-transparent border-none focus:ring-0 text-sm py-4 px-5 min-h-[60px] max-h-48 resize-none placeholder:text-zinc-600 text-zinc-200 focus-within:border-zinc-700 focus:border-0 focus:outline-0"
                ></textarea>

                <div class="flex items-center justify-between px-3 pb-2 pt-1">
                    <div class="flex items-center gap-1">
                        <input 
                            type="file" 
                            bind:this={fileInput} 
                            onchange={handleFileSelect} 
                            class="hidden" 
                            accept="image/*,audio/*,video/*,.pdf,.doc,.docx,.txt"
                        />
                        <button 
                            onclick={() => fileInput?.click()}
                            class="p-2 text-zinc-500 hover:text-zinc-300 transition-colors"
                        >
                            <Paperclip class="w-4 h-4"/>
                        </button>
                        <button class="p-2 text-zinc-500 hover:text-zinc-300 transition-colors">
                            <Sparkles class="w-4 h-4"/>
                        </button>
                    </div>

                    <button
                            onclick={handleSubmit}
                            disabled={(!inputMessage.trim() && !selectedFile) || chatStore.loading || isUploading}
                            class="flex items-center gap-2 px-4 py-2 rounded-xl bg-zinc-100 text-zinc-950 text-xs font-bold hover:bg-zinc-200 disabled:opacity-20 transition-all shadow-lg"
                    >
                        {#if isUploading}
                            <Loader2 class="w-3.5 h-3.5 animate-spin"/>
                            Uploading...
                        {:else}
                            Send
                            <Send class="w-3.5 h-3.5"/>
                        {/if}
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
