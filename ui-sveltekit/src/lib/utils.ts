import {createHighlighter} from "shiki";
import MarkdownIt from "markdown-it";
import {full as emoji} from "markdown-it-emoji";
import anchor from "markdown-it-anchor";

type EventHandler = (data: any) => void;

class EventBus {
	private subscribers: Map<string, Set<EventHandler>> = new Map();

	subscribe(event: string, handler: EventHandler) {
		if (!this.subscribers.has(event)) {
			this.subscribers.set(event, new Set());
		}
		this.subscribers.get(event)?.add(handler);
		return () => this.unsubscribe(event, handler);
	}

	unsubscribe(event: string, handler: EventHandler) {
		this.subscribers.get(event)?.delete(handler);
	}

	emit(event: string, data: any) {
		this.subscribers.get(event)?.forEach((handler) => handler(data));
	}
}

export const eventBus = new EventBus();

export function setupMarkdownHelpers() {
    if (typeof window === 'undefined') return;

    if (!(window as any).copyToClipboard) {
        (window as any).copyToClipboard = (btn: HTMLButtonElement) => {
            const code = decodeURIComponent(btn.getAttribute('data-code') || '');
            navigator.clipboard.writeText(code).then(() => {
                const originalInner = btn.innerHTML;
                btn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="text-emerald-500"><polyline points="20 6 9 17 4 12"/></svg>`;
                btn.classList.add('copied');
                setTimeout(() => {
                    btn.innerHTML = originalInner;
                    btn.classList.remove('copied');
                }, 2000);
            });
        };
    }

    if (!(window as any).toggleCodeBlock) {
        (window as any).toggleCodeBlock = (btn: HTMLButtonElement) => {
            const pre = btn.closest('pre');
            if (pre) {
                pre.classList.toggle('collapsed');
            }
        };
    }
}

export function formatTokenCount(tokens: number | string | undefined): string {
    const num = typeof tokens === 'string' ? parseInt(tokens) : (tokens ?? 0);
    if (isNaN(num)) return '0';
    
    if (num >= 10000000) {
        const suffixes = ['', 'K', 'M', 'B', 'T'];
        const suffixNum = Math.floor(("" + num).length / 3);
        let shortValue: number | string = parseFloat((suffixNum != 0 ? (num / Math.pow(1000, suffixNum)) : num).toPrecision(3));
        if (shortValue % 1 != 0) {
            shortValue = shortValue.toFixed(1);
        }
        return shortValue + suffixes[suffixNum];
    }
    
    return num.toLocaleString('de-DE'); // Use German locale for dot separator
}

export function useAvatar(name: string) {
    return `https://api.dicebear.com/7.x/avataaars/svg?seed=${encodeURIComponent(name)}`;
}

export function formatDate(date: string | Date | undefined): string {
    if (!date) return '';
    const d = typeof date === 'string' ? new Date(date) : date;
    if (isNaN(d.getTime())) return '';
    
    return d.toLocaleString('en-US', {
        hour: 'numeric',
        minute: 'numeric',
        hour12: true,
        month: 'short',
        day: 'numeric'
    });
}


export const highlighter  = await createHighlighter({
	themes: ['github-dark'],
	langs: ['javascript', 'typescript', 'rust', 'python', 'html', 'css', 'json', 'bash', 'sql']
});

export const mdIt = new MarkdownIt({
	html: true,
	linkify: true,
	typographer: true,
	highlight: (code, lang):string => {
		if (lang === 'mermaid') {
			return `<pre class="mermaid m-4 p-4">${code}</pre>`;
		}

		// @ts-ignore
		const isLoaded = lang && highlighter.getLoadedLanguages().includes(lang);
		let highlighted;

		if (isLoaded) {
			// Use codeToTokens to get the internal tokens and render them manually to avoid nested <pre>
            //@ts-ignore
			const tokens = highlighter.codeToTokens(code, { lang, theme: 'github-dark' });
			const linesHtml = tokens.tokens.map(line => {
				return `<span class="line">${line.map(token => {
					const style = `color: ${token.color || 'inherit'}`;
					return `<span style="${style}">${mdIt?.utils.escapeHtml(token.content)}</span>`;
				}).join('')}</span>`;
			}).join('\n');
			highlighted = `<pre class="shiki github-dark"><code>${linesHtml}</code></pre>`;
		} else {
			highlighted = `<pre class="shiki github-dark"><code>${mdIt?.utils.escapeHtml(code)}</code></pre>`;
		}

		const languageName = lang || 'code';
		const headerHtml = `
	<div class="code-block-header">
	<span class="code-lang">${languageName}</span>
	<div class="code-header-actions">
	<button 
	class="toggle-button"
	onclick="window.toggleCodeBlock(this)"
	title="Toggle code"
	>
	<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="toggle-icon"><path d="m6 9 6 6 6-6"/></svg>
	</button>
	<button 
	class="copy-button"
	data-code="${encodeURIComponent(code)}"
	onclick="window.copyToClipboard(this)"
	title="Copy code"
	>
	<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="copy-icon"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
	</button>
	</div>
	</div>`.trim();

		// Replace the inner pre tag with our styled version and header
		return highlighted
			.replace(/<pre([^>]*)>/, `<pre$1 style="position: relative;">${headerHtml}`);
	}

		}).use(emoji).use(anchor, {
		permalink: anchor.permalink.headerLink()
		});