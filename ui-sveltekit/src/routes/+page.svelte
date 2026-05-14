<script lang="ts">
    import {
        Zap,
        Shield,
        Cpu,
        Database,
        ArrowRight,
        CheckCircle2,
        CreditCard,
        Activity,
        Brain,
        Smartphone,
        Lock,
        MessageSquare,
        Camera,
        Mic
    } from 'lucide-svelte';
    import { onMount } from 'svelte';
    import { fade, fly, slide } from 'svelte/transition';
    import { chatApi } from '$lib/api/client';

    let email = $state('');
    let status = $state<'idle' | 'loading' | 'success' | 'error'>('idle');
    let message = $state('');

    async function handleJoinWaitlist() {
        if (!email) return;
        status = 'loading';
        try {
            const response = await chatApi.joinWaitlist(email);
            status = 'success';
            message = "You're on the list! We'll reach out soon.";
            email = '';
        } catch (err: any) {
            status = 'error';
            message = err.message || "Something went wrong. Try again.";
        }
    }

    // Mock Chat Sequence
    let chatStep = $state(0);
    const chatSequence = [
        { type: 'user', content: 'Logged 150k at Kopi Kenangan! ☕', icon: Camera },
        { type: 'nomi', content: 'Got it! Logged 150,000 IDR to Finance. 📉', icon: MessageSquare },
        { type: 'user', content: 'Remind me to service the CB150R at 5.', icon: Mic },
        { type: 'nomi', content: 'Got it! Reminder set for 5:00 PM WIB. 🏍️💨', icon: MessageSquare }
    ];

    onMount(() => {
        const interval = setInterval(() => {
            chatStep = (chatStep + 1) % (chatSequence.length + 1);
        }, 3000);
        return () => clearInterval(interval);
    });
</script>

<div class="min-h-screen bg-[#0f172a] text-slate-200 font-sans selection:bg-blue-500/30">
    <!-- Sticky Header -->
    <header class="fixed top-0 w-full z-50 border-b border-slate-800/50 bg-[#0f172a]/80 backdrop-blur-md">
        <div class="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
            <div class="flex items-center gap-2">
                <div class="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center">
                    <Zap class="w-5 h-5 text-white fill-white" />
                </div>
                <span class="text-xl font-black tracking-tighter text-white">NOMI</span>
            </div>
            <nav class="hidden md:flex items-center gap-8">
                <a href="#features" class="text-sm font-medium text-slate-400 hover:text-white transition-colors">Features</a>
                <a href="#stack" class="text-sm font-medium text-slate-400 hover:text-white transition-colors">Stack</a>
                <a href="/login" class="px-4 py-2 rounded-lg bg-slate-800 text-sm font-bold text-white hover:bg-slate-700 transition-all border border-slate-700">
                    Login
                </a>
            </nav>
        </div>
    </header>

    <main>
        <!-- Hero Section -->
        <section class="pt-32 pb-20 px-6 overflow-hidden">
            <div class="max-w-7xl mx-auto grid lg:grid-cols-2 gap-16 items-center">
                <div class="space-y-8" in:fly={{ y: 20, duration: 800 }}>
                    <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-blue-500/10 border border-blue-500/20">
                        <span class="relative flex h-2 w-2">
                            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75"></span>
                            <span class="relative inline-flex rounded-full h-2 w-2 bg-blue-500"></span>
                        </span>
                        <span class="text-[10px] font-black uppercase tracking-widest text-blue-400 font-mono">v2.0 Beta Now Open</span>
                    </div>

                    <h1 class="text-5xl md:text-7xl font-black text-white leading-[1.1] tracking-tight">
                        Your Life, <span class="text-transparent bg-clip-text bg-gradient-to-r from-blue-500 to-blue-400">Decoded.</span>
                    </h1>

                    <p class="text-lg md:text-xl text-slate-400 leading-relaxed max-w-xl">
                        The multimodal life infrastructure that lives where you do. Finance, vitality, and memories—all synced through a single, intelligent interface.
                    </p>
                    <div class="flex flex-col sm:flex-row gap-4">
                        <div class="relative flex-1 max-w-md">
                            <input
                                bind:value={email}
                                type="email"
                                placeholder="Enter your email"
                                class="w-full h-14 bg-slate-900 border border-slate-800 rounded-2xl px-6 text-white focus:ring-2 focus:ring-blue-500/50 outline-none transition-all"
                            />
                            <button
                                onclick={handleJoinWaitlist}
                                disabled={status === 'loading'}
                                class="absolute right-2 top-2 h-10 px-6 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white font-bold rounded-xl transition-all flex items-center gap-2 shadow-lg shadow-blue-500/20"
                            >
                                {#if status === 'loading'}
                                    <div class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                                {:else}
                                    Join Beta <ArrowRight class="w-4 h-4" />
                                {/if}
                            </button>
                        </div>
                    </div>

                    {#if message}
                        <p class="text-sm font-medium {status === 'success' ? 'text-emerald-400' : 'text-red-400'}" transition:slide>
                            {message}
                        </p>
                    {/if}
                </div>

                <!-- Floating Chat Mockup -->
                <div class="relative lg:h-[600px] flex items-center justify-center">
                    <div class="absolute inset-0 bg-gradient-to-tr from-blue-500/10 to-emerald-500/10 blur-3xl rounded-full"></div>

                    <div class="relative w-full max-w-[400px] space-y-4">
                        {#each chatSequence as item, i}
                            {#if i < chatStep}
                                <div
                                    in:fly={{ y: 20, duration: 500 }}
                                    class="flex {item.type === 'user' ? 'justify-end' : 'justify-start'}"
                                >
                                    <div class="max-w-[85%] p-4 rounded-2xl border transition-all {item.type === 'user' ? 'bg-blue-600 border-blue-500 text-white shadow-xl shadow-blue-900/20' : 'bg-slate-900 border-slate-800 text-slate-200'}">
                                        <div class="flex items-center gap-3 mb-1">
                                            <item.icon class="w-3.5 h-3.5 opacity-60" />
                                            <span class="text-[10px] font-bold uppercase tracking-widest opacity-60">
                                                {item.type === 'user' ? 'You' : 'Nomi'}
                                            </span>
                                        </div>
                                        <p class="text-sm font-medium leading-relaxed">{item.content}</p>
                                    </div>
                                </div>
                            {/if}
                        {/each}
                    </div>
                </div>
            </div>
        </section>

        <!-- Feature Matrix -->
        <section id="features" class="py-24 px-6 bg-slate-900/50">
            <div class="max-w-7xl mx-auto">
                <div class="text-center mb-20 space-y-4">
                    <h2 class="text-3xl md:text-5xl font-black text-white tracking-tight">One brain. Multiple domains.</h2>
                    <p class="text-slate-400 max-w-2xl mx-auto">Nomi integrates deeply with your existing digital life, transforming raw data into actionable insights.</p>
                </div>

                <div class="grid md:grid-cols-3 gap-8">
                    <!-- Finance -->
                    <div class="group p-8 bg-slate-900 border border-slate-800 rounded-3xl hover:border-blue-500/50 transition-all">
                        <div class="w-12 h-12 bg-blue-500/10 rounded-2xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                            <CreditCard class="w-6 h-6 text-blue-400" />
                        </div>
                        <h3 class="text-xl font-bold text-white mb-4">Finance: Snap & Log</h3>
                        <p class="text-slate-400 leading-relaxed mb-6">Send, Voice, or Snap. Instant SQL logging with S3 document backup. Your expenses, organized automatically.</p>
                        <ul class="space-y-3">
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-blue-500" /> Multi-currency support
                            </li>
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-blue-500" /> Visual receipt parsing
                            </li>
                        </ul>
                    </div>

                    <!-- Vitality -->
                    <div class="group p-8 bg-slate-900 border border-slate-800 rounded-3xl hover:border-emerald-500/50 transition-all">
                        <div class="w-12 h-12 bg-emerald-500/10 rounded-2xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                            <Activity class="w-6 h-6 text-emerald-400" />
                        </div>
                        <h3 class="text-xl font-bold text-white mb-4">Vitality: Live Sync</h3>
                        <p class="text-slate-400 leading-relaxed mb-6">Synced with your pulse via Samsung Health and Health Connect. Real-time biofeedback and habit tracking.</p>
                        <ul class="space-y-3">
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-emerald-500" /> Sleep quality analysis
                            </li>
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-emerald-500" /> Activity goal nudges
                            </li>
                        </ul>
                    </div>

                    <!-- Memories -->
                    <div class="group p-8 bg-slate-900 border border-slate-800 rounded-3xl hover:border-purple-500/50 transition-all">
                        <div class="w-12 h-12 bg-purple-500/10 rounded-2xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                            <Brain class="w-6 h-6 text-purple-400" />
                        </div>
                        <h3 class="text-xl font-bold text-white mb-4">Memories: Time-Travel</h3>
                        <p class="text-slate-400 leading-relaxed mb-6">Leveraging pgvector and RAG. Recall any conversation, document, or event by context or date.</p>
                        <ul class="space-y-3">
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-purple-500" /> Semantic search
                            </li>
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-purple-500" /> Multi-modal recall
                            </li>
                        </ul>
                    </div>
                </div>
            </div>
        </section>

        <!-- Tech Stack -->
        <section id="stack" class="py-24 px-6 overflow-hidden">
            <div class="max-w-7xl mx-auto">
                <div class="grid lg:grid-cols-2 gap-16 items-center">
                    <div class="space-y-8">
                        <h2 class="text-3xl md:text-5xl font-black text-white tracking-tight">Engineered for Trust.</h2>
                        <p class="text-lg text-slate-400 leading-relaxed">
                            Built with the most reliable technologies to ensure your data is safe, accessible, and processed at lightning speeds.
                        </p>

                        <div class="grid grid-cols-2 gap-6">
                            <div class="flex items-center gap-4">
                                <div class="w-10 h-10 rounded-xl bg-orange-500/10 flex items-center justify-center border border-orange-500/20">
                                    <Cpu class="w-5 h-5 text-orange-400" />
                                </div>
                                <div>
                                    <p class="text-sm font-bold text-white">Rust</p>
                                    <p class="text-[10px] text-slate-500 uppercase tracking-widest font-bold">Safety & Speed</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-4">
                                <div class="w-10 h-10 rounded-xl bg-blue-500/10 flex items-center justify-center border border-blue-500/20">
                                    <MessageSquare class="w-5 h-5 text-blue-400" />
                                </div>
                                <div>
                                    <p class="text-sm font-bold text-white">Gemini</p>
                                    <p class="text-[10px] text-slate-500 uppercase tracking-widest font-bold">Intelligence</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-4">
                                <div class="w-10 h-10 rounded-xl bg-orange-400/10 flex items-center justify-center border border-orange-400/20">
                                    <Zap class="w-5 h-5 text-orange-300" />
                                </div>
                                <div>
                                    <p class="text-sm font-bold text-white">SvelteKit</p>
                                    <p class="text-[10px] text-slate-500 uppercase tracking-widest font-bold">Responsive UI</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-4">
                                <div class="w-10 h-10 rounded-xl bg-blue-600/10 flex items-center justify-center border border-blue-600/20">
                                    <Database class="w-5 h-5 text-blue-500" />
                                </div>
                                <div>
                                    <p class="text-sm font-bold text-white">PostgreSQL</p>
                                    <p class="text-[10px] text-slate-500 uppercase tracking-widest font-bold">Reliable Storage</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="relative">
                        <div class="absolute -inset-10 bg-blue-500/10 blur-3xl rounded-full"></div>
                        <div class="relative p-8 bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl">
                            <div class="flex items-center gap-3 mb-8 pb-8 border-b border-slate-800">
                                <Lock class="w-5 h-5 text-blue-400" />
                                <span class="text-sm font-bold text-white uppercase tracking-widest">End-to-End Security</span>
                            </div>
                            <div class="space-y-6">
                                <div class="h-2 w-full bg-slate-800 rounded-full overflow-hidden">
                                    <div class="h-full w-3/4 bg-blue-500 rounded-full animate-pulse"></div>
                                </div>
                                <div class="h-2 w-full bg-slate-800 rounded-full overflow-hidden">
                                    <div class="h-full w-1/2 bg-emerald-500 rounded-full animate-pulse"></div>
                                </div>
                                <div class="h-2 w-full bg-slate-800 rounded-full overflow-hidden">
                                    <div class="h-full w-[90%] bg-blue-500 rounded-full animate-pulse"></div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>

        <!-- Final CTA -->
        <section class="py-32 px-6">
            <div class="max-w-4xl mx-auto text-center p-12 md:p-20 bg-gradient-to-br from-blue-600 to-blue-800 rounded-[3rem] shadow-2xl shadow-blue-900/40 relative overflow-hidden">
                <div class="absolute top-0 right-0 p-10 opacity-10">
                    <Zap class="w-64 h-64 text-white fill-white" />
                </div>
                <div class="relative z-10 space-y-8">
                    <h2 class="text-4xl md:text-6xl font-black text-white tracking-tight">Ready to Decode?</h2>
                    <p class="text-blue-100 text-lg max-w-xl mx-auto">Join the early beta and start building your multimodal life infrastructure today.</p>
                    <div class="flex flex-col sm:flex-row justify-center gap-4">
                        <div class="relative flex-1 max-w-md mx-auto sm:mx-0">
                            <input
                                bind:value={email}
                                type="email"
                                placeholder="Your email address"
                                class="w-full h-16 bg-white/10 border border-white/20 rounded-2xl px-6 text-white placeholder:text-white/60 focus:ring-2 focus:ring-white/50 outline-none transition-all"
                            />
                            <button
                                onclick={handleJoinWaitlist}
                                disabled={status === 'loading'}
                                class="absolute right-2 top-2 h-12 px-8 bg-white text-blue-600 hover:bg-blue-50 disabled:opacity-50 font-black rounded-xl transition-all shadow-xl"
                            >
                                Get Started
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    </main>

    <footer class="py-12 px-6 border-t border-slate-800/50">
        <div class="max-w-7xl mx-auto flex flex-col md:flex-row justify-between items-center gap-8">
            <div class="flex items-center gap-2 grayscale opacity-50">
                <Zap class="w-5 h-5 text-white fill-white" />
                <span class="text-lg font-black tracking-tighter text-white uppercase">NOMI</span>
            </div>
            <p class="text-slate-500 text-sm font-medium">© 2026 Arta AI Orchestrator. All rights reserved.</p>
            <div class="flex gap-6">
                <Smartphone class="w-5 h-5 text-slate-600" />
                <MessageSquare class="w-5 h-5 text-slate-600" />
                <Shield class="w-5 h-5 text-slate-600" />
            </div>
        </div>
    </footer>
</div>

<style>
    :global(html) {
        scroll-behavior: smooth;
    }

    @keyframes pulse-subtle {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.8; }
    }
</style>
