<script lang="ts">
	import { popupStore } from '$lib/stores/popup.svelte';
	import Popup from './Popup.svelte';
</script>

{#each popupStore.popups as popup (popup.id)}
	<div style="z-index: {100 + popupStore.popups.indexOf(popup)}">
		<Popup {popup} />
	</div>
{/each}
