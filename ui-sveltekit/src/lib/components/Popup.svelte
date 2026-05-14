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
	class="fixed inset-0 z-[100] bg-black/50 backdrop-blur-sm"
	transition:fade={{ duration: 200 }}
	onclick={handleOutsideClick}
>
	<!-- Popup Container -->
	<div
		class="absolute inset-y-0 right-0 flex w-full flex-col bg-[#0f172a] shadow-2xl border-l border-slate-800 text-slate-100 {popup.width} max-w-1/2"
		transition:fly={{ x: '100%', duration: 300, opacity: 1 }}
	>
		<!-- Header -->
		<header class="flex h-14 items-center justify-between border-b border-slate-800 px-6 bg-[#0f172a]/80 backdrop-blur-md">
			{#if popup.headerSnippet}
				{@render popup.headerSnippet()}
			{:else}
				<h2 class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-200">
					{popup.title}
				</h2>
			{/if}
			<button
				type="button"
				class="rounded-lg p-2 text-slate-500 hover:bg-slate-900 hover:text-slate-300 transition-colors"
				onclick={() => popupStore.close(popup.id)}
			>
				<X size={18} />
			</button>
		</header>

		<!-- Content -->
		<main class="flex-1 overflow-y-auto px-6 py-6 custom-scrollbar">
			{@render popup.contentSnippet()}
		</main>

		<!-- Footer -->
		{#if popup.footerSnippet}
			<footer class="border-t border-slate-800 px-6 py-4 bg-[#0f172a]">
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
