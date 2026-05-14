<script lang="ts">
    import { FileText, ArrowLeft } from 'lucide-svelte';
    import { fly } from 'svelte/transition';

    let locale = $state<'en' | 'id'>('en');

    const content = {
        en: {
            title: "Terms & Conditions",
            lastUpdated: "Last updated: May 14, 2026",
            intro: "By using Nomi, you agree to the following terms. Please read them carefully.",
            sections: [
                {
                    title: "1. Acceptance of Terms",
                    body: "By accessing or using Nomi's services via WhatsApp or our web interface, you agree to be bound by these terms."
                },
                {
                    title: "2. User Conduct",
                    body: "You agree not to use Nomi for any unlawful purposes or to send harmful, offensive, or inappropriate content."
                },
                {
                    title: "3. Service Availability",
                    body: "While we strive for 100% uptime, Nomi is provided 'as is'. We may modify or suspend services at any time for maintenance or updates."
                }
            ],
            back: "Back to Home"
        },
        id: {
            title: "Syarat & Ketentuan",
            lastUpdated: "Terakhir diperbarui: 14 Mei 2026",
            intro: "Dengan menggunakan Nomi, Anda menyetujui persyaratan berikut. Harap baca dengan saksama.",
            sections: [
                {
                    title: "1. Penerimaan Ketentuan",
                    body: "Dengan mengakses atau menggunakan layanan Nomi melalui WhatsApp atau antarmuka web kami, Anda setuju untuk terikat oleh ketentuan ini."
                },
                {
                    title: "2. Perilaku Pengguna",
                    body: "Anda setuju untuk tidak menggunakan Nomi untuk tujuan yang melanggar hukum atau mengirim konten yang berbahaya, menyinggung, atau tidak pantas."
                },
                {
                    title: "3. Ketersediaan Layanan",
                    body: "Meskipun kami berusaha untuk ketersediaan 100%, Nomi disediakan 'apa adanya'. Kami dapat mengubah atau menghentikan layanan kapan saja untuk pemeliharaan atau pembaruan."
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
                    <FileText class="w-6 h-6 text-white" />
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
