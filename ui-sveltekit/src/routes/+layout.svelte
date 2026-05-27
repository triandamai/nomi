<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { chatApi } from '$lib/api/client';
	import { mqttClient } from '$lib/api/mqtt';
	import PopupManager from '$lib/components/PopupManager.svelte';
	import DiscordSidebar from "$lib/components/DiscordSidebar.svelte"
	import Header from '$lib/components/Header.svelte';
	import { conversationStore } from '$lib/stores/conversation.svelte';

	import { goto, beforeNavigate, afterNavigate } from '$app/navigation';
	import { page } from '$app/state';
	import {eventBus} from "$lib/utils";
	import {getSession} from "$lib/stores/profile.svelte";
	import { Toaster } from 'svelte-french-toast';

	let { children } = $props();

	let opening = false;

	async function open() {
		const [token] = getSession();
		const isPublicRoute = page.url.pathname === '/' || page.url.pathname === '/login' || page.url.pathname.startsWith('/docs');

		if (!token && !isPublicRoute) {
			goto('/login');
			return;
		}

		if (token && !isPublicRoute) {
			if (opening) return;
			opening = true;

			try {
				await conversationStore.loadConversations();
				// Initialize MQTT connection
				mqttClient.connect();
			} finally {
				opening = false;
			}
		}
	}

	onMount(() => {
		open();
		const unsubscribe = eventBus.subscribe("load", open);
		return () => {
			unsubscribe();
			mqttClient.disconnect();
		};
	});

	afterNavigate(() => {
		open();
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="dark flex h-[100dvh] bg-[--bg-main] text-[--text-main] font-sans selection:bg-zinc-800">
	{#if page.url.pathname !== '/login' && page.url.pathname !== '/'  && !page.url.pathname.startsWith('/docs')}
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

<Toaster />
