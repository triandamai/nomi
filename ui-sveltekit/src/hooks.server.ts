import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	// Since we are using sessionStorage (client-side only), 
    // the server cannot verify the session for SSR.
    // Dashboard protection will happen in the browser (+layout.svelte or onMount).
	return resolve(event);
};
