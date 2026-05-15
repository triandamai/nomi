<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { chatApi } from '$lib/api/client';
	import PopupManager from '$lib/components/PopupManager.svelte';
	import DiscordSidebar from "$lib/components/DiscordSidebar.svelte"
	import Header from '$lib/components/Header.svelte';
	import { conversationStore } from '$lib/stores/conversation.svelte';

	import { goto, beforeNavigate } from '$app/navigation';
	import { page } from '$app/state';
	import {eventBus} from "$lib/utils";
	import {ragStore} from "$lib/stores/rag.svelte";

	let { children } = $props();

	let closing: (() => void) | null = null;

	function open() {
		const token = sessionStorage.getItem('auth_token');
		const isPublicRoute = page.url.pathname === '/' || page.url.pathname === '/login';

		if (!token && !isPublicRoute) {
			goto('/login');
			return;
		}

		if (token && !isPublicRoute) {
			// Close existing connection if any
			if (closing) closing();

			conversationStore.loadConversations().finally(() => {
				ragStore.fetchGraph(conversationStore.activeConversationId);
				closing = chatApi.streamEvent();
			});
		}
	}

	onMount(() => {
		open();
		const unsubscribe = eventBus.subscribe("load", open);
		return () => {
			unsubscribe();
			if (closing) closing();
		};
	});

	beforeNavigate(() => {
		if (closing) {
			closing();
			closing = null;
		}
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="dark flex h-screen bg-[--bg-main] text-[--text-main] font-sans selection:bg-zinc-800">
	{#if page.url.pathname !== '/login' && page.url.pathname !== '/'}
		<DiscordSidebar />
		<div class="flex-1 flex flex-col relative overflow-hidden">
			<Header />
			{@render children()}
		</div>
	{:else}
		<div class="flex-1 overflow-y-auto">
			{@render children()}
		</div>
	{/if}
</div>

{#if page.url.pathname !== '/login'}
	<PopupManager />
{/if}
