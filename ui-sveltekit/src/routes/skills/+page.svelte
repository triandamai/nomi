<script lang="ts">
    import { onMount } from 'svelte';
    import { chatApi } from '$lib/api/client';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { tick } from 'svelte';
    import { mdIt, setupMarkdownHelpers } from '$lib/utils';
    import { 
        Wrench, 
        Zap, 
        Cpu, 
        Search, 
        Loader2, 
        ChevronRight, 
        Sparkles,
        Puzzle,
        Settings,
        Activity,
        CreditCard,
        MessageSquare,
        Image as ImageIcon,
        Mic,
        FileText,
        Globe,
        Calendar,
        Bell,
        Hash,
        Box,
        Info,
        ShieldCheck,
        Terminal,
        Code2,
        Brackets,
        User
    } from 'lucide-svelte';

    type Skill = {
        name: string;
        description: string;
        intents: string[];
        skill_type: 'System' | 'Dynamic';
        script_code?: string;
        schema_json?: any;
        creator_name?: string;
    };

    let skills = $state<Skill[]>([]);
    let isLoading = $state(true);
    let searchQuery = $state('');
    let selectedSkill = $state<Skill | null>(null);

    let showCode = $state(false);
    let showSchema = $state(false);

    const filteredSkills = $derived(
        skills.filter(s => 
            s.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
            s.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
            s.intents.some(i => i.toLowerCase().includes(searchQuery.toLowerCase()))
        )
    );

    const getIcon = (name: string) => {
        const n = name.toLowerCase();
        if (n.includes('finance') || n.includes('money')) return CreditCard;
        if (n.includes('health') || n.includes('vitality')) return Activity;
        if (n.includes('media') || n.includes('vision')) return ImageIcon;
        if (n.includes('audio') || n.includes('voice')) return Mic;
        if (n.includes('web') || n.includes('search')) return Globe;
        if (n.includes('remind') || n.includes('schedule') || n.includes('task')) return Bell;
        if (n.includes('doc') || n.includes('file') || n.includes('knowledge')) return FileText;
        if (n.includes('soul') || n.includes('personality')) return Sparkles;
        if (n.includes('message') || n.includes('chat')) return MessageSquare;
        if (n.includes('sticker')) return ImageIcon;
        if (n.includes('forecast')) return Globe;
        if (n.includes('sql') || n.includes('query')) return Box;
        return Wrench;
    };

    async function loadSkills() {
        isLoading = true;
        try {
            const res = await chatApi.getPublicSkills();
            skills = res.data;
        } catch (e) {
            console.error('Failed to load skills:', e);
        } finally {
            isLoading = false;
        }
    }

    async function openSkillDetail(skill: Skill) {
        selectedSkill = skill;
        showCode = false;
        showSchema = false;
        popupStore.open({
            title: 'Skill Intelligence',
            width: 'max-w-2xl',
            contentSnippet: skillDetailSnippet
        });
    }

    $effect(() => {
        if (mdIt && (showCode || showSchema || selectedSkill)) {
            tick().then(() => {
                setupMarkdownHelpers();
            });
        }
    });

    onMount(() => {
        loadSkills();
        setupMarkdownHelpers();
    });
</script>

{#snippet skillDetailSnippet()}
    {#if selectedSkill}
        {@const Icon = getIcon(selectedSkill.name)}
        <div class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-300 max-h-[75vh] overflow-y-auto pr-2 custom-scrollbar">
            <div class="flex items-start gap-6">
                <div class="p-5 bg-slate-950 rounded-2xl border border-slate-800 shadow-2xl">
                    <Icon class="w-8 h-8 text-blue-400" />
                </div>
                <div class="space-y-2">
                    <div class="flex items-center gap-3">
                        <span class="px-2 py-0.5 rounded-md text-[10px] font-black uppercase tracking-widest border {selectedSkill.skill_type === 'System' ? 'bg-blue-500/10 text-blue-400 border-blue-500/20' : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'}">
                            {selectedSkill.skill_type} Plugin
                        </span>
                        {#if selectedSkill.creator_name}
                            <div class="flex items-center gap-1.5 text-slate-500">
                                <User class="w-3 h-3" />
                                <span class="text-[10px] font-bold uppercase tracking-widest">Created by {selectedSkill.creator_name}</span>
                            </div>
                        {/if}
                    </div>
                    <h2 class="text-3xl font-black text-white tracking-tighter uppercase">{selectedSkill.name.replace(/_/g, ' ')}</h2>
                </div>
            </div>

            <div class="space-y-3">
                <div class="flex items-center gap-2 text-slate-500 px-1">
                    <Info class="w-3.5 h-3.5" />
                    <span class="text-[10px] font-bold uppercase tracking-widest">Functional Description</span>
                </div>
                <div class="p-5 bg-slate-950/50 border border-slate-800 rounded-2xl">
                    <p class="text-slate-300 leading-relaxed italic text-sm">
                        "{selectedSkill.description}"
                    </p>
                </div>
            </div>

            <div class="space-y-3">
                <div class="flex items-center gap-2 text-slate-500 px-1">
                    <Terminal class="w-3.5 h-3.5" />
                    <span class="text-[10px] font-bold uppercase tracking-widest">Inference Intents</span>
                </div>
                <div class="flex flex-wrap gap-2">
                    {#each selectedSkill.intents as intent}
                        <div class="flex items-center gap-2 px-3 py-1.5 bg-slate-900 border border-slate-800 rounded-xl">
                            <Hash class="w-3 h-3 text-blue-500" />
                            <span class="text-[10px] font-mono text-slate-400">{intent}</span>
                        </div>
                    {/each}
                </div>
            </div>

            {#if selectedSkill.skill_type === 'Dynamic' && selectedSkill.script_code}
                <div class="space-y-3">
                    <button 
                        onclick={() => showCode = !showCode}
                        class="flex items-center justify-between w-full p-4 bg-slate-900 border border-slate-800 rounded-2xl hover:border-emerald-500/50 transition-all group"
                    >
                        <div class="flex items-center gap-3 text-emerald-400">
                            <Code2 class="w-4 h-4" />
                            <span class="text-xs font-bold uppercase tracking-widest">Source Implementation</span>
                        </div>
                        <ChevronRight class="w-4 h-4 text-slate-600 transition-transform {showCode ? 'rotate-90' : ''}" />
                    </button>
                    
                    {#if showCode}
                        <div class="prose prose-invert max-w-none prose-sm overflow-hidden animate-in zoom-in-95 duration-200">
                            {@html mdIt?.render('```typescript\n' + selectedSkill.script_code + '\n```')}
                        </div>
                    {/if}
                </div>
            {/if}

            {#if selectedSkill.schema_json}
                <div class="space-y-3">
                    <button 
                        onclick={() => showSchema = !showSchema}
                        class="flex items-center justify-between w-full p-4 bg-slate-900 border border-slate-800 rounded-2xl hover:border-blue-500/50 transition-all group"
                    >
                        <div class="flex items-center gap-3 text-blue-400">
                            <Brackets class="w-4 h-4" />
                            <span class="text-xs font-bold uppercase tracking-widest">Tool Specification</span>
                        </div>
                        <ChevronRight class="w-4 h-4 text-slate-600 transition-transform {showSchema ? 'rotate-90' : ''}" />
                    </button>
                    
                    {#if showSchema}
                        <div class="prose prose-invert max-w-none prose-sm overflow-hidden animate-in zoom-in-95 duration-200">
                            {@html mdIt?.render('```json\n' + JSON.stringify(selectedSkill.schema_json, null, 2) + '\n```')}
                        </div>
                    {/if}
                </div>
            {/if}

            <div class="p-4 rounded-xl bg-blue-500/5 border border-blue-500/10 flex items-start gap-3">
                <ShieldCheck class="w-4 h-4 text-blue-500 mt-0.5" />
                <p class="text-[10px] text-slate-500 leading-relaxed">
                    This skill is verified and integrated into Nomi's core orchestration layer. It can be triggered automatically based on intent classification or manual discovery.
                </p>
            </div>
        </div>
    {/if}
{/snippet}

<div class="min-h-screen bg-[#0f172a] text-slate-200 overflow-y-auto">
    <!-- Header Area -->
    <header class="pt-20 pb-8 px-6">
        <div class="max-w-7xl mx-auto space-y-6">
            <div class="space-y-3">
                <div class="inline-flex items-center gap-2 px-2.5 py-0.5 rounded-full bg-blue-500/10 border border-blue-500/20">
                    <Zap class="w-3 h-3 text-blue-400 fill-blue-400" />
                    <span class="text-[9px] font-black uppercase tracking-widest text-blue-400">Capability Registry</span>
                </div>
                <h1 class="text-3xl md:text-5xl font-black text-white tracking-tighter uppercase">
                    Nomi's <span class="text-transparent bg-clip-text bg-gradient-to-r from-blue-500 to-emerald-400">Toolkit</span>
                </h1>
                <p class="text-slate-400 max-w-xl text-sm md:text-base leading-relaxed">
                    Explore the library of system-integrated skills and dynamic edge plugins that power Nomi's intelligence.
                </p>
            </div>

            <!-- Search Bar -->
            <div class="relative max-w-lg group">
                <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                    <Search class="w-4 h-4 text-slate-500 group-focus-within:text-blue-500 transition-colors" />
                </div>
                <input 
                    type="text" 
                    bind:value={searchQuery}
                    placeholder="Search skills, intents..." 
                    class="w-full pl-11 pr-4 py-3 bg-slate-900/50 border border-slate-800 rounded-xl focus:border-blue-500/50 focus:ring-1 focus:ring-blue-500/20 text-sm text-slate-100 placeholder:text-slate-600 outline-none transition-all backdrop-blur-xl"
                />
            </div>
        </div>
    </header>

    <main class="px-6 pb-20">
        <div class="max-w-7xl mx-auto">
            {#if isLoading}
                <div class="flex flex-col items-center justify-center py-20 space-y-4">
                    <Loader2 class="w-10 h-10 text-blue-500 animate-spin" />
                    <p class="text-[10px] font-bold uppercase tracking-widest text-slate-500">Synchronizing Skills...</p>
                </div>
            {:else if filteredSkills.length === 0}
                <div class="text-center py-20 space-y-4">
                    <p class="text-slate-500 text-sm font-medium">No skills found matching your search.</p>
                </div>
            {:else}
                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                    {#each filteredSkills as skill}
                        {@const Icon = getIcon(skill.name)}
                        <button 
                            onclick={() => openSkillDetail(skill)}
                            class="group p-5 bg-slate-900/40 border border-slate-800 rounded-2xl hover:border-blue-500/50 transition-all hover:bg-slate-900/60 relative overflow-hidden backdrop-blur-sm flex flex-col h-full text-left w-full"
                        >
                            <div class="absolute -top-12 -right-12 w-24 h-24 bg-blue-500/5 blur-3xl rounded-full group-hover:bg-blue-500/10 transition-all"></div>
                            
                            <div class="relative z-10 flex flex-col h-full w-full">
                                <div class="flex items-start justify-between mb-4">
                                    <div class="p-2.5 bg-slate-950 rounded-xl border border-slate-800 group-hover:border-blue-500/30 transition-all">
                                        <Icon class="w-5 h-5 text-blue-400" />
                                    </div>
                                    <span class="px-2 py-0.5 rounded-md text-[8px] font-black uppercase tracking-widest border {skill.skill_type === 'System' ? 'bg-blue-500/10 text-blue-400 border-blue-500/20' : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'}">
                                        {skill.skill_type}
                                    </span>
                                </div>

                                <h3 class="text-sm font-bold text-white mb-2 group-hover:text-blue-300 transition-colors uppercase tracking-tight truncate">
                                    {skill.name.replace(/_/g, ' ')}
                                </h3>
                                
                                <p class="text-xs text-slate-400 leading-relaxed mb-4 line-clamp-3 flex-1">
                                    {skill.description}
                                </p>

                                <div class="space-y-2 pt-4 border-t border-slate-800/50">
                                    <div class="flex flex-wrap gap-1.5">
                                        {#each skill.intents.slice(0, 3) as intent}
                                            <span class="px-1.5 py-0.5 bg-slate-950/50 border border-slate-800/50 rounded-md text-[8px] font-mono text-slate-500">
                                                {intent}
                                            </span>
                                        {/each}
                                        {#if skill.intents.length > 3}
                                            <span class="px-1.5 py-0.5 text-[8px] font-mono text-slate-600">
                                                +{skill.intents.length - 3}
                                            </span>
                                        {/if}
                                    </div>
                                </div>
                            </div>
                        </button>
                    {/each}
                </div>
            {/if}
        </div>
    </main>

    <!-- Footer Decoration -->
    <div class="h-24 bg-gradient-to-t from-blue-500/5 to-transparent"></div>
</div>

<style>
    :global(body) {
        background-color: #0f172a;
    }

    .custom-scrollbar::-webkit-scrollbar {
        width: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: transparent;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: #1e293b;
        border-radius: 10px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: #334155;
    }
</style>
