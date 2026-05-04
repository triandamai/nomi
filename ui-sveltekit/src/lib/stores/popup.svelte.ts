import { type Snippet } from 'svelte';

export interface PopupOptions {
	id?: string;
	title?: string;
	width?: string;
	closeOnOutsideClick?: boolean;
	headerSnippet?: Snippet;
	contentSnippet: Snippet;
	footerSnippet?: Snippet;
}

export interface PopupState extends PopupOptions {
	id: string;
	isOpen: boolean;
}

function createPopupStore() {
	let popups = $state<PopupState[]>([]);

	return {
		get popups() {
			return popups;
		},

		open(options: PopupOptions) {
			const id = options.id || Math.random().toString(36).substring(2, 9);
			const newPopup: PopupState = {
				id,
				title: options.title || '',
				width: options.width || 'max-w-md',
				closeOnOutsideClick: options.closeOnOutsideClick ?? true,
				headerSnippet: options.headerSnippet,
				contentSnippet: options.contentSnippet,
				footerSnippet: options.footerSnippet,
				isOpen: true
			};

			popups = [...popups, newPopup];
			return id;
		},

		close(id: string) {
			popups = popups.filter((p) => p.id !== id);
		},

		closeLast() {
			if (popups.length > 0) {
				popups = popups.slice(0, -1);
			}
		}
	};
}

export const popupStore = createPopupStore();
