<script lang="ts">
    import { Shield, ArrowLeft } from 'lucide-svelte';
    import { fly } from 'svelte/transition';

    let locale = $state<'en' | 'id'>('en');

    const content = {
        en: {
            title: "Privacy Policy",
            lastUpdated: "Last updated: May 14, 2026",
            intro: "At Nomi, we take your privacy seriously. This policy explains how we handle your data.",
            sections: [
                {
                    title: "1. Data Collection",
                    body: "We collect information you provide directly to Nomi via WhatsApp or our web interface, including messages, images, and health data if synced."
                },
                {
                    title: "2. How We Use Data",
                    body: "Your data is used to provide intelligent insights, reminders, and context-aware assistance. We use pgvector for semantic memory to help you recall your life events."
                },
                {
                    title: "3. Data Security",
                    body: "We implement advanced encryption and industry-standard security measures. Your data is stored in our secure Rust-based infrastructure."
                }
            ],
            back: "Back to Home"
        },
        id: {
            title: "Kebijakan Privasi",
            lastUpdated: "Terakhir diperbarui: 14 Mei 2026",
            intro: "Di Nomi, kami menjaga privasi Anda dengan serius. Kebijakan ini menjelaskan bagaimana kami menangani data Anda.",
            sections: [
                {
                    title: "1. Pengumpulan Data",
                    body: "Kami mengumpulkan informasi yang Anda berikan langsung ke Nomi melalui WhatsApp atau antarmuka web kami, termasuk pesan, gambar, dan data kesehatan jika disinkronkan."
                },
                {
                    title: "2. Cara Kami Menggunakan Data",
                    body: "Data Anda digunakan untuk memberikan wawasan cerdas, pengingat, dan bantuan berbasis konteks. Kami menggunakan pgvector untuk memori semantik guna membantu Anda mengingat peristiwa hidup Anda."
                },
                {
                    title: "3. Keamanan Data",
                    body: "Kami menerapkan enkripsi tingkat lanjut dan tindakan keamanan standar industri. Data Anda disimpan dalam infrastruktur berbasis Rust kami yang aman."
                }
            ],
            back: "Kembali ke Beranda"
        }
    };

    const t = $derived(content[locale]);
</script>

<div class="min-h-screen bg-[#0f172a] text-slate-200 font-sans p-6 md:p-12">
    <div class="max-w-3xl mx-auto space-y-12" in:fly={{ y: 20, duration: 800 }}>
        <header class="flex items-center justify-between border-b border-slate-800 pb-8">
            <div class="flex items-center gap-4">
                <div class="w-12 h-12 bg-blue-600 rounded-xl flex items-center justify-center">
                    <Shield class="w-6 h-6 text-white" />
                </div>
                <div>
                    <h1 class="text-3xl font-black text-white tracking-tight">{t.title}</h1>
                    <p class="text-sm text-slate-500">{t.lastUpdated}</p>
                </div>
            </div>
            
            <div class="flex items-center bg-slate-900/50 border border-slate-800 rounded-lg p-1">
                <button onclick={() => locale = 'en'} class="px-3 py-1 text-xs font-bold rounded-md {locale === 'en' ? 'bg-blue-600 text-white' : 'text-slate-500'}">EN</button>
                <button onclick={() => locale = 'id'} class="px-3 py-1 text-xs font-bold rounded-md {locale === 'id' ? 'bg-blue-600 text-white' : 'text-slate-500'}">ID</button>
            </div>
        </header>

        <div class="prose prose-invert max-w-none space-y-8 text-slate-400 leading-relaxed">
            <p class="text-lg text-slate-300">{t.intro}</p>

            {#each t.sections as section}
                <section class="space-y-4">
                    <h2 class="text-xl font-bold text-white">{section.title}</h2>
                    <p>{section.body}</p>
                </section>
            {/each}
        </div>

        <footer class="pt-12 border-t border-slate-800">
            <a href="/" class="inline-flex items-center gap-2 text-blue-400 hover:text-blue-300 font-bold transition-colors">
                <ArrowLeft class="w-4 h-4" />
                {t.back}
            </a>
        </footer>
    </div>
</div>
