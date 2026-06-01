<script lang="ts">
	import { popupStore } from '$lib/stores/popup.svelte';
	import { fly, fade } from 'svelte/transition';
	import { X } from 'lucide-svelte';

	let { popup } = $props<{ popup: any }>();

	function handleOutsideClick(e: MouseEvent) {
		if (popup.closeOnOutsideClick && e.target === e.currentTarget) {
			popupStore.close(popup.id);
		}
	}
</script>

<!-- Overlay -->
<div
	role="button"
	class="fixed inset-0 z-[100] bg-black/40 backdrop-blur-[6px] transition-all duration-300"
	transition:fade={{ duration: 250 }}
	onclick={handleOutsideClick}
>
	<!-- Popup Container -->
	<div
		class="absolute inset-y-0 right-0 flex w-full flex-col bg-slate-950/65 border-l border-slate-800/30 shadow-[0_0_60px_-15px_rgba(0,0,0,0.5)] text-slate-100 {popup.width} max-w-full md:max-w-[85%] lg:max-w-1/2 backdrop-blur-3xl"
		transition:fly={{ x: '100%', duration: 350, opacity: 1 }}
	>
		<!-- Header -->
		<header class="flex h-14 min-h-[3.5rem] items-center justify-between border-b border-slate-800/30 px-6 bg-slate-950/30 backdrop-blur-md sticky top-0 z-10">
			{#if popup.headerSnippet}
				{@render popup.headerSnippet()}
			{:else}
				<h2 class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-200">
					{popup.title}
				</h2>
			{/if}
			<button
				type="button"
				class="rounded-xl p-2 text-slate-400 hover:bg-slate-800/50 hover:text-white transition-all duration-250"
				onclick={() => popupStore.close(popup.id)}
			>
				<X size={18} />
			</button>
		</header>

		<!-- Content -->
		<main class="flex-1 overflow-y-auto px-6 custom-scrollbar bg-transparent">
			{@render popup.contentSnippet()}
            <!-- Bottom spacer to ensure scrolling feels free and unbound -->
            <div class="h-20 w-full pointer-events-none"></div>
		</main>

		<!-- Footer -->
		{#if popup.footerSnippet}
			<footer class="border-t border-slate-800/30 px-6 py-4 bg-slate-950/45 backdrop-blur-md">
				{@render popup.footerSnippet()}
			</footer>
		{/if}
	</div>
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
