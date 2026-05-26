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
        Wrench,
        Reply,
        Code
    } from 'lucide-svelte';
    import {chatStore, type Message} from '$lib/stores/chat.svelte';
    import {conversationStore} from '$lib/stores/conversation.svelte';
    import {chatApi} from '$lib/api/client';
    import {formatTokenCount, formatDate} from '$lib/utils';
    import ToolResult from '$lib/components/ToolResult.svelte';
    import ChatBubble from '$lib/components/ChatBubble.svelte';
    import PricingPopUp from '$lib/components/PricingPopUp.svelte';
    import {onMount, tick} from 'svelte';
    import {eventBus} from "$lib/utils";
    import {afterNavigate, goto} from '$app/navigation';
    import {popupStore} from '$lib/stores/popup.svelte';

    let inputMessage = $state('');
    let scrollContainer = $state<HTMLElement | null>(null);
    let isNearBottom = true;
    let fileInput = $state<HTMLInputElement | null>(null);
    let selectedFile = $state<File | null>(null);
    let isUploading = $state(false);

    let textareaRef = $state<HTMLTextAreaElement | null>(null);
    let members = $state<Array<{ user_id: string; display_name: string | null; external_id: string | null; channel_type: string | null }>>([]);
    let showMentions = $state(false);
    let mentionQuery = $state('');
    let mentionIndex = $state(0);
    let triggerIndex = $state(-1);
    let selectedMentions = $state<any[]>([]);
    let globalUsers = $state<any[]>([]);

    $effect(() => {
        const cid = conversationStore.activeConversationId;
        if (cid) {
            chatApi.getConversationMembers(cid).then(res => {
                if (res && res.data) {
                    members = res.data;
                }
            }).catch(err => console.error("Failed to load members", err));
        } else {
            members = [];
        }
    });

    let searchTimeout: any;
    $effect(() => {
        const query = mentionQuery.trim();
        if (showMentions && query.length >= 1) {
            if (searchTimeout) clearTimeout(searchTimeout);
            searchTimeout = setTimeout(() => {
                chatApi.searchUsers(query).then(res => {
                    if (res && res.data) {
                        globalUsers = res.data;
                    }
                }).catch(err => console.error("Failed to search global users", err));
            }, 150);
        } else {
            globalUsers = [];
        }
    });

    let filteredMembers = $derived(() => {
        if (!members) return [];
        const query = mentionQuery.toLowerCase();
        
        // Local channel members matching query
        const localMatches = members.filter(m => 
            m.external_id && 
            (!query || 
             (m.display_name && m.display_name.toLowerCase().includes(query)) || 
             m.external_id.toLowerCase().includes(query))
        );

        // Global users matching query
        const globalMatches = globalUsers.filter(g => 
            g.external_id && 
            (!query || 
             (g.display_name && g.display_name.toLowerCase().includes(query)) || 
             g.external_id.toLowerCase().includes(query))
        );

        // Merge and deduplicate by external_id
        const merged = [...localMatches];
        for (const g of globalMatches) {
            if (!merged.some(m => m.external_id === g.external_id)) {
                merged.push(g);
            }
        }

        return merged;
    });

    function checkMentionTrigger() {
        if (!textareaRef) return;
        const text = inputMessage;
        const cursorPos = textareaRef.selectionStart;
        
        const lastAtIdx = text.lastIndexOf('@', cursorPos - 1);
        if (lastAtIdx === -1) {
            showMentions = false;
            return;
        }

        if (lastAtIdx > 0 && text[lastAtIdx - 1] !== ' ' && text[lastAtIdx - 1] !== '\n') {
            showMentions = false;
            return;
        }

        const textBetween = text.substring(lastAtIdx + 1, cursorPos);
        if (textBetween.includes(' ') || textBetween.includes('\n')) {
            showMentions = false;
            return;
        }

        showMentions = true;
        mentionQuery = textBetween;
        triggerIndex = lastAtIdx;
        mentionIndex = 0;
    }

    function selectMention(member: any) {
        if (!textareaRef || triggerIndex === -1) return;
        const text = inputMessage;
        const cursorPos = textareaRef.selectionStart;
        
        let rawId = member.external_id || '';
        if (rawId.includes('@')) {
            rawId = rawId.split('@')[0];
        }

        const nameToInsert = member.display_name || rawId;
        const mentionText = `@${nameToInsert} `;

        // Track selected mention for raw ID translation upon sending
        if (!selectedMentions.some(m => m.external_id === member.external_id)) {
            selectedMentions.push(member);
        }

        const before = text.substring(0, triggerIndex);
        const after = text.substring(cursorPos);
        
        inputMessage = before + mentionText + after;
        showMentions = false;
        
        const newCursorPos = triggerIndex + mentionText.length;
        tick().then(() => {
            if (textareaRef) {
                textareaRef.focus();
                textareaRef.setSelectionRange(newCursorPos, newCursorPos);
            }
        });
    }

    let backdropRef = $state<HTMLDivElement | null>(null);

    function handleTextareaScroll(e: Event) {
        const target = e.target as HTMLTextAreaElement;
        if (backdropRef) {
            backdropRef.scrollTop = target.scrollTop;
            backdropRef.scrollLeft = target.scrollLeft;
        }
    }

    function formatComposerHTML(text: string) {
        if (!text) return '<span class="text-slate-600">Message Nomi...</span>';

        // Escape HTML
        let escaped = text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');

        // Highlight Mentions: @username/id in a distinct blue color with 0 character width deviation!
        escaped = escaped.replace(/@([a-zA-Z0-9_\-]+)/g, (match, username) => {
            const member = members.find(m => 
                (m.display_name && m.display_name.toLowerCase() === username.toLowerCase()) || 
                (m.external_id && m.external_id.toLowerCase().includes(username.toLowerCase()))
            );
            if (member) {
                // Return styled text with exactly the same characters and widths! No extra paddings or borders!
                return `<span class="text-blue-400 font-bold">${match}</span>`;
            }
            return match;
        });

        // Add trailing newline placeholder
        if (text.endsWith('\n')) {
            escaped += '&nbsp;';
        }

        return escaped;
    }

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

        let processedMsg = inputMessage;
        for (const mention of selectedMentions) {
            if (mention.display_name && mention.external_id) {
                let rawId = mention.external_id;
                if (rawId.includes('@')) {
                    rawId = rawId.split('@')[0];
                }
                const escapedName = mention.display_name.replace(/[-\/\\^$*+?.()|[\]{}]/g, '\\$&');
                processedMsg = processedMsg.replace(new RegExp(`@${escapedName}\\b`, 'g'), `@${rawId}`);
            }
        }

        const msg = processedMsg;
        inputMessage = '';
        selectedFile = null;
        selectedMentions = [];

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
        if (showMentions && filteredMembers().length > 0) {
            if (e.key === 'ArrowDown') {
                e.preventDefault();
                mentionIndex = (mentionIndex + 1) % filteredMembers().length;
                return;
            }
            if (e.key === 'ArrowUp') {
                e.preventDefault();
                mentionIndex = (mentionIndex - 1 + filteredMembers().length) % filteredMembers().length;
                return;
            }
            if (e.key === 'Enter' || e.key === 'Tab') {
                e.preventDefault();
                selectMention(filteredMembers()[mentionIndex]);
                return;
            }
            if (e.key === 'Escape') {
                e.preventDefault();
                showMentions = false;
                return;
            }
        }

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
        afterNavigate(() => {
            chatStore.fetchMessages(false).finally(() => {
                console.log("finish load messages")
            })
        })
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
        class="whatsapp-canvas flex-1 overflow-y-auto px-6 pt-8 {selectedFile ? 'pb-64' : 'pb-48'} space-y-10 scroll-smooth"
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
        <div class="max-w-4xl mx-auto space-y-1">
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
                {@const prevMessage: Message | undefined | null = chatStore.messages[index - 1]}
                <div class="group flex gap-6 animate-in fade-in slide-in-from-bottom-4 duration-500 {msg.user_id !== prevMessage?.user_id ? 'mt-10':'mt-0'}">
                    <div class="flex-shrink-0 pt-1">
                        {#if (msg.role === 'user' && msg.user_id !== prevMessage?.user_id)}
                            <div class="w-8 h-8 rounded-lg bg-slate-100 flex items-center justify-center">
                                <User class="w-4 h-4 text-slate-950"/>
                            </div>
                        {:else if msg.role === 'assistant' || msg.role === 'system'}
                            <div class="w-8 h-8 rounded-lg bg-slate-900 border border-slate-800 flex items-center justify-center">
                                <Bot class="w-4 h-4 text-slate-400"/>
                            </div>
                        {:else}
                            <div class="w-8 h-8 rounded-lg bg-transparent flex items-center justify-center"></div>
                        {/if}
                    </div>

                    <div class="flex-1 flex flex-col min-w-0 space-y-4">
                        {#if (msg.user_id !== prevMessage?.user_id)}
                            <div class="flex items-center justify-between">
                                <div class="flex items-center gap-2">
                                <span class="text-xs font-bold uppercase tracking-wider {msg.role === 'user' ? 'text-slate-300' : 'text-blue-400'}">
                                    {msg.role === 'user' ? msg.display_name ?? 'Human' : 'Nomi'}
                                </span>
                                    {#if msg.role !== 'user'}
                                    <span class="font-mono text-[10px] text-slate-500">- {formatTokenCount(msg.total_tokens)}
                                        Token</span>
                                    {/if}
                                </div>
                                {#if msg.created_at}
                                <span class="text-[10px] font-mono text-slate-500 uppercase tracking-tight">
                                    {formatDate(msg.created_at)}
                                </span>
                                {/if}
                            </div>
                        {/if}

                        {#if msg.toolCalls && msg.toolCalls.length > 0}
                            <div class="space-y-3">
                                {#each msg.toolCalls as tc}
                                    <ToolResult args="" tool={tc.tool} result={tc.result}/>
                                {/each}
                            </div>
                        {/if}

                        <div class="relative group/bubble flex flex-col">
                            <ChatBubble
                                    content={msg.content}
                                    thought={msg.thought}
                                    image_url={msg.image_url}
                                    video_url={msg.video_url}
                                    audio_url={msg.audio_url}
                                    document_url={msg.document_url}
                                    sticker_url={msg.sticker_url}
                                    metadata={msg.metadata}
                                    replied_message={msg.replied_message}
                                    onToggleThought={(expanded: boolean) => handleToggleThought(expanded, index === chatStore.messages.length - 1)}
                            />

                            <!-- Message Actions: Standardized Bottom Row for Reachability -->
                            <div class="flex items-center gap-2 mt-2 opacity-0 group-hover/bubble:opacity-100 transition-opacity">
                                <button
                                    onclick={() => { chatStore.replyingToMessage = msg; }}
                                    class="flex items-center gap-1.5 px-3 py-1.5 bg-slate-900/40 hover:bg-slate-800/60 border border-slate-800 rounded-full text-[9px] font-black uppercase tracking-widest text-slate-500 hover:text-blue-400 hover:border-blue-500/30 transition-all shadow-lg backdrop-blur-sm"
                                    title="Reply to message"
                                >
                                    <Reply class="w-3 h-3" />
                                    <span>Reply</span>
                                </button>
                            </div>
                        </div>
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

                {#if chatStore.replyingToMessage}
                    <div class="px-4 md:px-5 pt-4 pb-2">
                        <div class="flex items-center justify-between p-3 bg-blue-500/5 border border-blue-500/20 rounded-xl animate-in fade-in slide-in-from-bottom-2 duration-300">
                            <div class="flex items-center gap-3 min-w-0">
                                <div class="p-2 bg-blue-500/10 rounded-lg">
                                    <Reply class="w-3.5 h-3.5 text-blue-400" />
                                </div>
                                <div class="min-w-0">
                                    <p class="text-[9px] font-black uppercase tracking-widest text-blue-400/80 mb-0.5">
                                        Replying to <span class="text-blue-300">{chatStore.replyingToMessage.display_name || chatStore.replyingToMessage.role || 'Human'}</span>
                                    </p>
                                    <p class="text-[11px] text-slate-400 truncate italic">
                                        {chatStore.replyingToMessage.content}
                                    </p>
                                </div>
                            </div>
                            <button 
                                onclick={() => { chatStore.replyingToMessage = null; }}
                                class="p-1.5 hover:bg-slate-800 rounded-lg text-slate-500 hover:text-white transition-colors"
                            >
                                <X class="w-3.5 h-3.5" />
                            </button>
                        </div>
                    </div>
                {/if}

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

                {#if showMentions && filteredMembers().length > 0}
                    <div class="absolute bottom-full left-0 right-0 mb-2 max-h-60 overflow-y-auto bg-slate-950/95 border border-slate-800 rounded-xl shadow-2xl backdrop-blur-2xl z-50 divide-y divide-slate-900 animate-in fade-in slide-in-from-bottom-2 duration-200">
                        {#each filteredMembers() as member, idx}
                            {@const rawId = member.external_id?.includes('@') ? member.external_id.split('@')[0] : member.external_id}
                            <button
                                type="button"
                                onclick={() => selectMention(member)}
                                class="w-full flex items-center gap-3 px-4 py-3 text-left transition-colors {idx === mentionIndex ? 'bg-blue-600/20 text-white border-l-2 border-blue-500' : 'text-slate-300 hover:bg-slate-900/50 hover:text-white'}"
                            >
                                <div class="w-7 h-7 rounded-lg bg-gradient-to-tr from-blue-600 to-indigo-600 flex items-center justify-center text-xs font-black text-white shadow-md">
                                    {member.display_name?.substring(0, 2).toUpperCase() || 'U'}
                                </div>
                                <div class="flex-1 min-w-0">
                                    <p class="text-xs font-bold truncate">{member.display_name || 'Anonymous'}</p>
                                    <p class="text-[10px] text-slate-500 truncate">@{rawId} • {member.channel_type || 'platform'}</p>
                                </div>
                            </button>
                        {/each}
                    </div>
                {/if}

                <div class="relative w-full min-h-[50px] md:min-h-[60px] max-h-32 md:max-h-48 overflow-hidden text-sm leading-relaxed">
                    <!-- Synced Backdrop containing the rich styled HTML -->
                    <div
                        bind:this={backdropRef}
                        class="w-full min-h-[50px] md:min-h-[60px] max-h-32 md:max-h-48 py-3 md:py-4 px-4 md:px-5 overflow-y-auto whitespace-pre-wrap break-words pointer-events-none select-none text-slate-200 hide-scrollbar"
                        style="font-family: inherit; font-size: inherit; line-height: inherit;"
                    >
                        {@html formatComposerHTML(inputMessage)}
                    </div>

                    <!-- Interactive transparent textarea overlayed on top -->
                    <textarea
                        bind:this={textareaRef}
                        bind:value={inputMessage}
                        onkeydown={handleKeydown}
                        oninput={(e) => { checkMentionTrigger(); handleTextareaScroll(e); }}
                        onclick={checkMentionTrigger}
                        onscroll={handleTextareaScroll}
                        placeholder={showMentions ? '' : 'Message Nomi...'}
                        class="absolute inset-0 w-full h-full bg-transparent border-none focus:ring-0 text-transparent caret-blue-500 py-3 md:py-4 px-4 md:px-5 resize-none overflow-y-auto whitespace-pre-wrap break-words outline-none focus:outline-none focus:border-0 focus:ring-0"
                        style="font-family: inherit; font-size: inherit; line-height: inherit; color: transparent; -webkit-text-fill-color: transparent; caret-color: #3b82f6;"
                    ></textarea>
                </div>

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
    .whatsapp-canvas {
        background-color: #0f172a;
        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='80' height='80' viewBox='0 0 80 80'%3E%3Cg fill='%23202c33' fill-opacity='0.22'%3E%3Cpath d='M15 5h2v2h-2zm0 10h2v2h-2zm10-5h2v2h-2zm10 20h2v2h-2zm-20 10h2v2h-2zm30-5h2v2h-2zM5 45h2v2h-2zm15 15h2v2h-2zm40-30h2v2h-2zm-10-10h2v2h-2zm10 30h2v2h-2zm-20 15h2v2h-2zm30 10h2v2h-2zM55 5h2v2h-2zm0 10h2v2h-2zm-40 50h2v2h-2zm30 10h2v2h-2zm10-25h2v2h-2zm-5 15h2v2h-2zm-25 5h2v2h-2zm-10-35h2v2h-2z'/%3E%3Ccircle cx='40' cy='40' r='1'/%3E%3Cpath d='M45 40c0-2.8 2.2-5 5-5s5 2.2 5 5-2.2 5-5 5-5-2.2-5-5zm-30 0c0-2.8 2.2-5 5-5s5 2.2 5 5-2.2 5-5 5-5-2.2-5-5z'/%3E%3C/g%3E%3C/svg%3E");
        background-repeat: repeat;
        background-size: 140px 140px;
    }

    :global(body) {
        background-color: #0f172a;
        margin: 0;
        height: 100dvh;
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
    .hide-scrollbar::-webkit-scrollbar {
        display: none !important;
    }
    .hide-scrollbar {
        -ms-overflow-style: none !important;  /* IE and Edge */
        scrollbar-width: none !important;  /* Firefox */
    }
</style>
