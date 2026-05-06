import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	// For simplicity in this prototype, we'll handle auth client-side, 
    // but a production app should check cookies here.
	return resolve(event);
};
