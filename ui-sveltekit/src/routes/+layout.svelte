<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { chatApi } from '$lib/api/client';
	import PopupManager from '$lib/components/PopupManager.svelte';
	import DiscordSidebar from "$lib/components/DiscordSidebar.svelte"
	import Header from '$lib/components/Header.svelte';
	import { conversationStore } from '$lib/stores/conversation.svelte';

	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import {eventBus} from "$lib/utils";

	let { children } = $props();

	onMount(() => {
		const token = localStorage.getItem('auth_token');
		if (!token && page.url.pathname !== '/login') {
			goto('/login');
			return;
		}
		let closing:()=> void = ()=>{

		}

		function open(){
			if (token) {
				conversationStore.loadConversations();
				closing  = chatApi.streamEvent();
			}
		}
		open()
		eventBus.subscribe("load",open)
		return ()=>{
			eventBus.unsubscribe("load",open)
			closing()
		}
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="dark flex h-screen bg-[--bg-main] text-[--text-main] font-sans selection:bg-zinc-800">
	{#if page.url.pathname !== '/login'}
		<DiscordSidebar />
		<div class="flex-1 flex flex-col relative overflow-hidden">
			<Header />
			{@render children()}
		</div>
	{:else}
		<div class="flex-1">
			{@render children()}
		</div>
	{/if}
</div>

{#if page.url.pathname !== '/login'}
	<PopupManager />
{/if}
