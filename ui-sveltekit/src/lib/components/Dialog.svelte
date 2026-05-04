<script lang="ts">
	import { fade, scale } from 'svelte/transition';
	import { X } from 'lucide-svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		isOpen: boolean;
		onClose: () => void;
		title?: string;
		clickOutside?: boolean;
		children: Snippet;
		footer?: Snippet;
		maxWidth?: string;
		maxHeight?: string;
	}

	let {
		isOpen,
		onClose,
		title,
		clickOutside = false,
		children,
		footer,
		maxWidth = 'max-w-2xl',
		maxHeight = 'max-h-[85vh]'
	} = $props<Props>();

	function handleOutsideClick(e: MouseEvent) {
		if (clickOutside && e.target === e.currentTarget) {
			onClose();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (isOpen && e.key === 'Escape') {
			onClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<div
		class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
		transition:fade={{ duration: 200 }}
		onclick={handleOutsideClick}
		aria-modal="true"
		role="dialog"
	>
		<div
			class="flex w-full flex-col overflow-hidden rounded-xl bg-zinc-950 shadow-2xl ring-1 ring-zinc-800 transition-all {maxWidth} {maxHeight}"
			transition:scale={{ duration: 200, start: 0.95 }}
		>
			<!-- Header -->
			<header class="flex shrink-0 items-center justify-between border-b px-6 py-4 border-zinc-800 bg-zinc-950">
				{#if title}
					<h2 class="text-sm font-bold uppercase tracking-widest text-zinc-200">
						{title}
					</h2>
				{:else}
					<div></div>
				{/if}
				<button
					type="button"
					class="rounded-lg p-2 text-zinc-500 hover:bg-zinc-900 hover:text-zinc-300 transition-colors"
					onclick={onClose}
					aria-label="Close dialog"
				>
					<X size={18} />
				</button>
			</header>

			<!-- Content -->
			<main class="flex-1 overflow-y-auto px-6 py-6 custom-scrollbar bg-zinc-950">
				{@render children()}
			</main>

			<!-- Footer -->
			{#if footer}
				<footer class="shrink-0 border-t border-zinc-800 bg-zinc-900/50 px-6 py-4">
					{@render footer()}
				</footer>
			{/if}
		</div>
	</div>
{/if}

<style>
	.custom-scrollbar::-webkit-scrollbar {
		width: 6px;
	}
	.custom-scrollbar::-webkit-scrollbar-track {
		background: transparent;
	}
	.custom-scrollbar::-webkit-scrollbar-thumb {
		background: #3f3f46;
		border-radius: 10px;
	}
	.custom-scrollbar::-webkit-scrollbar-thumb:hover {
		background: #52525b;
	}
</style>
