<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { chatApi } from '$lib/api/client';
	import PopupManager from '$lib/components/PopupManager.svelte';
	import DiscordSidebar from "$lib/components/DiscordSidebar.svelte"

	let { children } = $props();

	onMount(() => {
		const close = chatApi.streamEvent();
		return () => close();
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="flex h-screen bg-[#09090b] text-zinc-100 font-sans selection:bg-zinc-800">
	<DiscordSidebar />
	<div class="flex-1 flex flex-col relative overflow-hidden">
		{@render children()}
	</div>
</div>

<PopupManager />
