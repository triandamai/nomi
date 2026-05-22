<script lang="ts">
	import { onMount } from 'svelte';
	import { chatApi } from '$lib/api/client';
	import { setupMarkdownHelpers, mdIt } from '$lib/utils';
	import { FileCode, Loader2, Sparkles } from 'lucide-svelte';

	let content = $state('');
	let loading = $state(true);

	onMount(async () => {
		setupMarkdownHelpers();
		try {
			const res = await chatApi.getSkillsDocumentation();
			if (res.data) {
				content = res.data;
			}
		} catch (e) {
			console.error('Failed to load skills documentation', e);
		} finally {
			loading = false;
		}
	});

	let renderedContent = $derived(mdIt.render(content));
</script>

<div class="flex flex-col h-screen bg-bg-main text-text-main overflow-hidden font-sans">
	<!-- Standard Header Alignment -->
	<header class="h-16 flex-shrink-0 flex items-center justify-between px-6 border-b border-border-main bg-bg-main/80 backdrop-blur-md z-10">
		<div class="flex items-center gap-3">
			<div class="p-2 bg-accent-emerald/10 rounded-lg border border-accent-emerald/20">
				<FileCode class="w-5 h-5 text-accent-emerald" />
			</div>
			<div>
				<h1 class="text-lg font-semibold tracking-tight text-text-main">Skill Creation Protocol</h1>
				<p class="text-xs text-text-muted font-medium uppercase tracking-[0.1em]">Technical standards for DAF autonomous engineering</p>
			</div>
		</div>

		<div class="flex items-center gap-2 bg-border-main px-3 py-1.5 rounded-full border border-accent-emerald/20 shadow-lg shadow-accent-emerald/5">
			<div class="w-2 h-2 bg-accent-emerald rounded-full animate-pulse shadow-[0_0_8px_#10b981]"></div>
			<span class="text-[10px] font-mono font-bold text-accent-emerald uppercase tracking-widest text-center italic">Protocol v1.0 Live</span>
		</div>
	</header>

	<main class="flex-1 overflow-y-auto custom-scrollbar bg-bg-main">
		<div class="max-w-4xl mx-auto py-12 px-6 pb-24">
			{#if loading}
				<div class="flex flex-col items-center justify-center py-32 gap-6 opacity-50">
					<Loader2 class="w-8 h-8 animate-spin text-accent-emerald" />
					<p class="text-sm font-mono tracking-[0.3em] uppercase text-text-muted">Fetching Protocol Definition...</p>
				</div>
			{:else}
				<div class="prose prose-invert prose-emerald max-w-none 
					prose-headings:text-text-main prose-headings:font-bold prose-headings:tracking-tight
					prose-p:text-text-muted prose-p:leading-relaxed prose-p:text-[15px]
					prose-code:text-accent-emerald prose-code:bg-accent-emerald/5 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:border prose-code:border-accent-emerald/20 prose-code:font-mono prose-code:before:content-[''] prose-code:after:content-['']
					prose-strong:text-text-main prose-strong:font-black
					prose-pre:bg-border-main/20 prose-pre:border prose-pre:border-border-main prose-pre:rounded-2xl prose-pre:shadow-2xl prose-pre:p-0
					animate-in fade-in slide-in-from-bottom-4 duration-700">
					{@html renderedContent}
				</div>

				<!-- Footer Hint for Nomi -->
				<div class="mt-20 pt-10 border-t border-border-main flex gap-6 items-start bg-border-main/10 p-8 rounded-3xl border-dashed relative overflow-hidden group">
                    <div class="absolute inset-0 bg-accent-emerald/[0.02] opacity-0 group-hover:opacity-100 transition-opacity"></div>
					<div class="p-3 bg-accent-emerald/10 rounded-xl text-accent-emerald border border-accent-emerald/20 relative z-10">
						<Sparkles class="w-6 h-6" />
					</div>
					<div class="relative z-10">
						<h4 class="text-sm font-black text-text-main uppercase tracking-[0.2em] mb-3">Note for SWE Agent</h4>
						<p class="text-[13px] text-text-muted leading-relaxed max-w-2xl font-medium">
							This document is your primary source of truth for plugin architecture. 
							When you are tasked with synthesizing new tools, refer to these standards to ensure 100% compatibility 
							with the Bun/V8 edge runtime and the Nomi Gateway v2.0 protocol.
						</p>
					</div>
				</div>
			{/if}
		</div>
	</main>
</div>

<style>
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
