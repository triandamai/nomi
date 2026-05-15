<script lang="ts">
    import {
        Zap,
        Shield,
        Cpu,
        Database,
        CheckCircle2,
        CreditCard,
        Activity,
        Brain,
        Smartphone,
        Lock,
        MessageSquare,
        Camera,
        Mic,
        Menu,
        X
    } from 'lucide-svelte';
    import {onMount} from 'svelte';
    import {fly, slide} from 'svelte/transition';
    import {env} from '$env/dynamic/public';

    const NOMI_WA = env.PUBLIC_NOMI_WA || '<PLACEHOLDER>';

    let mobileMenuOpen = $state(false);

    // Localization
    let locale = $state<'en' | 'id'>('en');

    const currentYear = new Date().getFullYear();

    const translations = {
        en: {
            nav: {
                features: "Features",
                stack: "Stack",
                login: "Login"
            },
            hero: {
                badge: "v2.0 Beta Now Open",
                title: "Your Life,",
                titleHighlight: "Decoded.",
                subtitle: "The multimodal life infrastructure that lives where you do. Finance, vitality, and memories—all synced through a single, intelligent interface."
            },
            features: {
                title: "One brain. Multiple domains.",
                subtitle: "Nomi integrates deeply with your existing digital life, transforming raw data into actionable insights.",
                finance: {
                    title: "Finance: Snap & Log",
                    desc: "Send, Voice, or Snap. Instant SQL logging with S3 document backup. Your expenses, organized automatically.",
                    list1: "Multi-currency support",
                    list2: "Visual receipt parsing"
                },
                vitality: {
                    title: "Vitality: Live Sync",
                    desc: "Synced with your pulse via Samsung Health and Health Connect. Real-time biofeedback and habit tracking.",
                    list1: "Sleep quality analysis",
                    list2: "Activity goal nudges"
                },
                memories: {
                    title: "Memories: Time-Travel",
                    desc: "Leveraging pgvector and RAG. Recall any conversation, document, or event by context or date.",
                    list1: "Semantic search",
                    list2: "Multi-modal recall"
                }
            },
            stack: {
                title: "Engineered for Trust.",
                subtitle: "Built with the most reliable technologies to ensure your data is safe, accessible, and processed at lightning speeds.",
                rust: "Safety & Speed",
                gemini: "Intelligence",
                svelte: "Responsive UI",
                postgres: "Reliable Storage",
                security: "Fast and Lightweight Agent"
            },
            finalCta: {
                title: "Ready to Decode?",
                subtitle: "Join the early beta and start building your multimodal life infrastructure today."
            },
            cta: "Start Chatting 🚀",
            footer: {
                rights: `@ ${currentYear} Build with ❤️ By Trian Damai`,
                language: "Language"
            },
            chat: [
                'Logged 150k at Kopi Kenangan! ☕',
                'Got it! Logged 150,000 IDR to Finance. 📉',
                'Remind me to service the AC at 5.',
                'Got it! Reminder set for 5:00 PM WIB. 🏍️💨'
            ]
        },
        id: {
            nav: {
                features: "Fitur",
                stack: "Teknologi",
                login: "Masuk"
            },
            hero: {
                badge: "v2.0 Beta Now Open",
                title: "Your Life,",
                titleHighlight: "Decoded.",
                subtitle: "Infrastruktur hidup multimodal yang hadir di mana pun kamu berada. Keuangan, vitalitas, dan kenangan—semuanya sinkron melalui satu antarmuka cerdas."
            },
            features: {
                title: "Satu otak. Berbagai domain.",
                subtitle: "Nomi terintegrasi secara mendalam dengan kehidupan digitalmu, mengubah data mentah menjadi wawasan berharga.",
                finance: {
                    title: "Keuangan: Snap & Log",
                    desc: "Kirim teks, suara, atau foto. Pencatatan SQL instan dengan cadangan dokumen S3. Pengeluaranmu, tertata otomatis.",
                    list1: "Dukungan multi-mata uang",
                    list2: "Pemindaian struk visual"
                },
                vitality: {
                    title: "Vitalitas: Sinkronisasi Langsung",
                    desc: "Sinkron dengan detak jantungmu melalui Samsung Health dan Health Connect. Biofeedback real-time dan pelacakan kebiasaan.",
                    list1: "Analisis kualitas tidur",
                    list2: "Dorongan target aktivitas"
                },
                memories: {
                    title: "Memori: Perjalanan Waktu",
                    desc: "Memanfaatkan pgvector dan RAG. Ingat kembali percakapan, dokumen, atau peristiwa apa pun berdasarkan konteks atau tanggal.",
                    list1: "Pencarian semantik",
                    list2: "Ingatan multi-modal"
                }
            },
            stack: {
                title: "Dirancang untuk Kepercayaan.",
                subtitle: "Dibangun dengan teknologi paling andal untuk memastikan datamu aman, mudah diakses, dan diproses dengan kecepatan cahaya.",
                rust: "Keamanan & Kecepatan",
                gemini: "Kecerdasan",
                svelte: "UI Responsif",
                postgres: "Penyimpanan Andal",
                security: "Agen Cepat & Ringan"
            },
            finalCta: {
                title: "Siap Bergabung?",
                subtitle: "Bergabunglah dengan beta awal dan mulai bangun infrastruktur hidup multimodal-mu hari ini."
            },
            cta: "Mulai Chatting 🚀",
            footer: {
                rights: `@ ${currentYear} Build with ❤️ By Trian Damai`,
                language: "Bahasa"
            },
            chat: [
                'Catat 150rb di Kopi Kenangan! ☕',
                'Siap! Berhasil mencatat 150.000 IDR ke Keuangan. 📉',
                'Ingatkan aku servis AC jam 5 sore.',
                'Siap! Pengingat disetel untuk jam 17:00 WIB. 🏍️💨'
            ]
        }
    };

    const t = $derived(translations[locale]);

    // Mock Chat Sequence
    let chatStep = $state(0);
    const chatSequence = $derived([
        {type: 'user', content: t.chat[0], icon: Camera},
        {type: 'nomi', content: t.chat[1], icon: MessageSquare},
        {type: 'user', content: t.chat[2], icon: Mic},
        {type: 'nomi', content: t.chat[3], icon: MessageSquare}
    ]);

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
                    <Zap class="w-5 h-5 text-white fill-white"/>
                </div>
                <span class="text-xl font-black tracking-tighter text-white">NOMI</span>
            </div>

            <!-- Desktop Nav -->
            <nav class="hidden md:flex items-center gap-8">
                <a href="#features"
                   class="text-sm font-medium text-slate-400 hover:text-white transition-colors">{t.nav.features}</a>
                <a href="#stack"
                   class="text-sm font-medium text-slate-400 hover:text-white transition-colors">{t.nav.stack}</a>

                <!-- Language Toggle -->
                <div class="flex items-center bg-slate-900/50 border border-slate-800 rounded-lg p-1">
                    <button
                            onclick={() => locale = 'en'}
                            class="px-2 py-1 text-[10px] font-bold rounded-md transition-all {locale === 'en' ? 'bg-blue-600 text-white shadow-sm' : 'text-slate-500 hover:text-slate-300'}"
                    >
                        EN
                    </button>
                    <button
                            onclick={() => locale = 'id'}
                            class="px-2 py-1 text-[10px] font-bold rounded-md transition-all {locale === 'id' ? 'bg-blue-600 text-white shadow-sm' : 'text-slate-500 hover:text-slate-300'}"
                    >
                        ID
                    </button>
                </div>

                <a href="/login"
                   class="px-4 py-2 rounded-lg bg-slate-800 text-sm font-bold text-white hover:bg-slate-700 transition-all border border-slate-700">
                    {t.nav.login}
                </a>
            </nav>

            <!-- Mobile Menu Toggle -->
            <button
                    class="md:hidden p-2 text-slate-400 hover:text-white"
                    onclick={() => mobileMenuOpen = !mobileMenuOpen}
            >
                {#if mobileMenuOpen}
                    <X class="w-6 h-6"/>
                {:else}
                    <Menu class="w-6 h-6"/>
                {/if}
            </button>
        </div>

        <!-- Mobile Nav -->
        {#if mobileMenuOpen}
            <div
                    class="md:hidden absolute top-16 left-0 w-full bg-[#0f172a] border-b border-slate-800 shadow-xl"
                    transition:slide
            >
                <nav class="flex flex-col p-6 gap-6">
                    <div class="flex flex-col gap-4">
                        <a
                                href="#features"
                                class="text-lg font-medium text-slate-400"
                                onclick={() => mobileMenuOpen = false}
                        >{t.nav.features}</a>
                        <a
                                href="#stack"
                                class="text-lg font-medium text-slate-400"
                                onclick={() => mobileMenuOpen = false}
                        >{t.nav.stack}</a>
                    </div>

                    <!-- Mobile Language Toggle -->
                    <div class="flex items-center gap-2">
                        <span class="text-sm font-bold text-slate-500 uppercase tracking-widest">{t.footer.language}</span>
                        <div class="flex items-center bg-slate-900/50 border border-slate-800 rounded-lg p-1 ml-auto">
                            <button
                                    onclick={() => locale = 'en'}
                                    class="px-4 py-2 text-xs font-bold rounded-md transition-all {locale === 'en' ? 'bg-blue-600 text-white shadow-sm' : 'text-slate-500 hover:text-slate-300'}"
                            >
                                EN
                            </button>
                            <button
                                    onclick={() => locale = 'id'}
                                    class="px-4 py-2 text-xs font-bold rounded-md transition-all {locale === 'id' ? 'bg-blue-600 text-white shadow-sm' : 'text-slate-500 hover:text-slate-300'}"
                            >
                                ID
                            </button>
                        </div>
                    </div>

                    <a
                            href="/login"
                            class="w-full py-4 rounded-xl bg-blue-600 text-center font-bold text-white"
                    >{t.nav.login}</a>
                </nav>
            </div>
        {/if}
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
                        <span class="text-[10px] font-black uppercase tracking-widest text-blue-400 font-mono">{t.hero.badge}</span>
                    </div>

                    <h1 class="text-5xl md:text-7xl font-black text-white leading-[1.1] tracking-tight">
                        {t.hero.title} <span
                            class="text-transparent bg-clip-text bg-gradient-to-r from-blue-500 to-blue-400">{t.hero.titleHighlight}</span>
                    </h1>

                    <p class="text-lg md:text-xl text-slate-400 leading-relaxed max-w-xl">
                        {t.hero.subtitle}
                    </p>

                    <!-- Responsive CTA Hero -->
                    <div class="flex flex-col gap-6">
                        <div class="flex flex-col gap-4">
                            <a
                                    href="{`https://wa.me/${NOMI_WA}?text=%2Fhelp`}"
                                    target="_blank"
                                    class="inline-flex h-16 px-10 bg-gradient-to-r from-blue-600 to-blue-500 hover:from-blue-500 hover:to-blue-400 text-white font-black rounded-2xl transition-all items-center justify-center gap-3 shadow-xl shadow-blue-500/20 text-lg group w-fit"
                            >
                                {t.cta}
                            </a>
                        </div>
                    </div>
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
                                            <item.icon class="w-3.5 h-3.5 opacity-60"/>
                                            <span class="text-[10px] font-bold uppercase tracking-widest opacity-60">
                                                {item.type === 'user' ? (locale === 'en' ? 'You' : 'Kamu') : 'Nomi'}
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
                    <h2 class="text-3xl md:text-5xl font-black text-white tracking-tight">{t.features.title}</h2>
                    <p class="text-slate-400 max-w-2xl mx-auto">{t.features.subtitle}</p>
                </div>

                <div class="grid md:grid-cols-3 gap-8">
                    <!-- Finance -->
                    <div class="group p-8 bg-slate-900 border border-slate-800 rounded-3xl hover:border-blue-500/50 transition-all">
                        <div class="w-12 h-12 bg-blue-500/10 rounded-2xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                            <CreditCard class="w-6 h-6 text-blue-400"/>
                        </div>
                        <h3 class="text-xl font-bold text-white mb-4">{t.features.finance.title}</h3>
                        <p class="text-slate-400 leading-relaxed mb-6">{t.features.finance.desc}</p>
                        <ul class="space-y-3">
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-blue-500"/> {t.features.finance.list1}
                            </li>
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-blue-500"/> {t.features.finance.list2}
                            </li>
                        </ul>
                    </div>

                    <!-- Vitality -->
                    <div class="group p-8 bg-slate-900 border border-slate-800 rounded-3xl hover:border-emerald-500/50 transition-all">
                        <div class="w-12 h-12 bg-emerald-500/10 rounded-2xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                            <Activity class="w-6 h-6 text-emerald-400"/>
                        </div>
                        <h3 class="text-xl font-bold text-white mb-4">{t.features.vitality.title}</h3>
                        <p class="text-slate-400 leading-relaxed mb-6">{t.features.vitality.desc}</p>
                        <ul class="space-y-3">
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-emerald-500"/> {t.features.vitality.list1}
                            </li>
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-emerald-500"/> {t.features.vitality.list2}
                            </li>
                        </ul>
                    </div>

                    <!-- Memories -->
                    <div class="group p-8 bg-slate-900 border border-slate-800 rounded-3xl hover:border-purple-500/50 transition-all">
                        <div class="w-12 h-12 bg-purple-500/10 rounded-2xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform">
                            <Brain class="w-6 h-6 text-purple-400"/>
                        </div>
                        <h3 class="text-xl font-bold text-white mb-4">{t.features.memories.title}</h3>
                        <p class="text-slate-400 leading-relaxed mb-6">{t.features.memories.desc}</p>
                        <ul class="space-y-3">
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-purple-500"/> {t.features.memories.list1}
                            </li>
                            <li class="flex items-center gap-2 text-sm text-slate-500">
                                <CheckCircle2 class="w-4 h-4 text-purple-500"/> {t.features.memories.list2}
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
                        <h2 class="text-3xl md:text-5xl font-black text-white tracking-tight">{t.stack.title}</h2>
                        <p class="text-lg text-slate-400 leading-relaxed">
                            {t.stack.subtitle}
                        </p>

                        <div class="grid grid-cols-2 gap-6">
                            <div class="flex items-center gap-4">
                                <div class="w-10 h-10 rounded-xl bg-orange-500/10 flex items-center justify-center border border-orange-500/20">
                                    <Cpu class="w-5 h-5 text-orange-400"/>
                                </div>
                                <div>
                                    <p class="text-sm font-bold text-white">Rust</p>
                                    <p class="text-[10px] text-slate-500 uppercase tracking-widest font-bold">{t.stack.rust}</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-4">
                                <div class="w-10 h-10 rounded-xl bg-blue-500/10 flex items-center justify-center border border-blue-500/20">
                                    <MessageSquare class="w-5 h-5 text-blue-400"/>
                                </div>
                                <div>
                                    <p class="text-sm font-bold text-white">Gemini</p>
                                    <p class="text-[10px] text-slate-500 uppercase tracking-widest font-bold">{t.stack.gemini}</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-4">
                                <div class="w-10 h-10 rounded-xl bg-orange-400/10 flex items-center justify-center border border-orange-400/20">
                                    <Zap class="w-5 h-5 text-orange-300"/>
                                </div>
                                <div>
                                    <p class="text-sm font-bold text-white">SvelteKit</p>
                                    <p class="text-[10px] text-slate-500 uppercase tracking-widest font-bold">{t.stack.svelte}</p>
                                </div>
                            </div>
                            <div class="flex items-center gap-4">
                                <div class="w-10 h-10 rounded-xl bg-blue-600/10 flex items-center justify-center border border-blue-600/20">
                                    <Database class="w-5 h-5 text-blue-500"/>
                                </div>
                                <div>
                                    <p class="text-sm font-bold text-white">PostgreSQL</p>
                                    <p class="text-[10px] text-slate-500 uppercase tracking-widest font-bold">{t.stack.postgres}</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="relative">
                        <div class="absolute -inset-10 bg-blue-500/10 blur-3xl rounded-full"></div>
                        <div class="relative p-8 bg-slate-900 border border-slate-800 rounded-3xl shadow-2xl">
                            <div class="flex items-center gap-3 mb-8 pb-8 border-b border-slate-800">
                                <Lock class="w-5 h-5 text-blue-400"/>
                                <span class="text-sm font-bold text-white uppercase tracking-widest">{t.stack.security}</span>
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
            <div class="max-w-4xl mx-auto text-center p-8 md:p-20 bg-gradient-to-br from-blue-600 to-blue-800 rounded-[2rem] md:rounded-[3rem] shadow-2xl shadow-blue-900/40 relative overflow-hidden">
                <div class="absolute top-0 right-0 p-10 opacity-10">
                    <Zap class="w-64 h-64 text-white fill-white"/>
                </div>
                <div class="relative z-10 space-y-8">
                    <h2 class="text-4xl md:text-6xl font-black text-white tracking-tight">{t.finalCta.title}</h2>
                    <p class="text-blue-100 text-lg max-w-xl mx-auto">{t.finalCta.subtitle}</p>
                    <div class="flex flex-col gap-6 items-center">
                        <a
                                href="{`https://wa.me/${NOMI_WA}?text=%2Fhelp`}"
                                target="_blank"
                                class="inline-flex h-16 px-10 bg-white text-blue-600 hover:bg-blue-50 text-lg font-black rounded-2xl transition-all items-center justify-center gap-3 shadow-xl shadow-blue-900/20 group w-full max-w-md"
                        >
                            {t.cta}
                        </a>
                    </div>
                </div>
            </div>
        </section>
    </main>

    <footer class="py-12 px-6 border-t border-slate-800/50">
        <div class="max-w-7xl mx-auto flex flex-col md:flex-row justify-between items-center gap-8">
            <div class="flex items-center gap-2 grayscale opacity-50">
                <Zap class="w-5 h-5 text-white fill-white"/>
                <span class="text-lg font-black tracking-tighter text-white uppercase">NOMI</span>
            </div>
            <p class="text-slate-500 text-sm font-medium">
                {@html t.footer.rights.replace('Trian Damai', '<a href="https://github.com/triandamai" target="_blank" class="hover:text-blue-400 transition-colors underline decoration-blue-500/30 underline-offset-4">Trian Damai</a>')}
            </p>
            <div class="flex gap-8 text-xs font-bold text-slate-500 uppercase tracking-widest">
                <a href="/privacy"
                   class="hover:text-blue-400 transition-colors">{locale === 'en' ? 'Privacy' : 'Privasi'}</a>
                <a href="/terms"
                   class="hover:text-blue-400 transition-colors">{locale === 'en' ? 'Terms' : 'Ketentuan'}</a>
            </div>
            <div class="flex gap-6">
                <Smartphone class="w-5 h-5 text-slate-600"/>
                <MessageSquare class="w-5 h-5 text-slate-600"/>
                <Shield class="w-5 h-5 text-slate-600"/>
            </div>
        </div>
    </footer>
</div>

<style>
    :global(html) {
        scroll-behavior: smooth;
    }

    @keyframes pulse-subtle {
        0%, 100% {
            opacity: 1;
        }
        50% {
            opacity: 0.8;
        }
    }
</style>
