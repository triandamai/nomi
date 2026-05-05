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
		class="absolute inset-y-0 right-0 flex w-full flex-col bg-[#09090b] shadow-2xl border-l border-zinc-800 text-zinc-100 {popup.width} max-w-1/2"
		transition:fly={{ x: '100%', duration: 300, opacity: 1 }}
	>
		<!-- Header -->
		<header class="flex h-14 items-center justify-between border-b border-zinc-800 px-6 bg-[#09090b]/80 backdrop-blur-md">
			{#if popup.headerSnippet}
				{@render popup.headerSnippet()}
			{:else}
				<h2 class="text-xs font-bold uppercase tracking-widest text-zinc-200">
					{popup.title}
				</h2>
			{/if}
			<button
				type="button"
				class="rounded-lg p-2 text-zinc-500 hover:bg-zinc-900 hover:text-zinc-300 transition-colors"
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
			<footer class="border-t border-zinc-800 px-6 py-4 bg-[#09090b]">
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
		background: #18181b;
		border-radius: 10px;
	}
	.custom-scrollbar::-webkit-scrollbar-thumb:hover {
		background: #27272a;
	}
</style>
