import { onMount } from 'svelte';

export interface ThemePalette {
    id: string;
    name: string;
    description: string;
    primaryColor: string;
    accentColor: string;
    bgPreview: string;
}

export const THEME_PALETTES: ThemePalette[] = [
    {
        id: 'slate-dark',
        name: 'Slate Midnight',
        description: 'Classic premium dark mode',
        primaryColor: '#3b82f6', // blue
        accentColor: '#10b981', // emerald
        bgPreview: '#0f172a'
    },
    {
        id: 'nord-frost',
        name: 'Nordic Frost',
        description: 'Clean Scandinavian cool breeze',
        primaryColor: '#88c0d0', // nord cyan
        accentColor: '#a3be8c', // nord light green
        bgPreview: '#2e3440'
    },
    {
        id: 'glass-purple',
        name: 'Amethyst Velvet',
        description: 'Rich deep violet glassmorphism',
        primaryColor: '#a855f7', // purple
        accentColor: '#ec4899', // pink
        bgPreview: '#120e2e'
    },
    {
        id: 'cyberpunk-neon',
        name: 'Cyberpunk 2077',
        description: 'High-contrast retro neon future',
        primaryColor: '#00ffff', // cyan
        accentColor: '#ff0055', // neon red
        bgPreview: '#0c0813'
    },
    {
        id: 'monokai-pro',
        name: 'Monokai Pro',
        description: 'Classic developer aesthetic',
        primaryColor: '#e6db74', // monokai yellow
        accentColor: '#a6e22e', // monokai green
        bgPreview: '#272822'
    },
    {
        id: 'emerald-forest',
        name: 'Forest Canopy',
        description: 'Organic calming deep green',
        primaryColor: '#10b981', // emerald
        accentColor: '#fbbf24', // amber
        bgPreview: '#064e3b'
    },
    {
        id: 'crystal-light',
        name: 'Crystal Light',
        description: 'Apple Crystal fluid glass light mode',
        primaryColor: '#3b82f6', // blue
        accentColor: '#ec4899', // pink
        bgPreview: '#f1f5f9'
    },
    {
        id: 'm3-expressive',
        name: 'M3 Expressive',
        description: 'Vibrant contrasting Material 3 palette',
        primaryColor: '#d0bcff', // light purple/lavender
        accentColor: '#ffb2be', // light pink/rose
        bgPreview: '#140f1f'
    },
    {
        id: 'sakura-blossom',
        name: 'Sakura Blossom',
        description: 'Delicate cherry blossom light mode',
        primaryColor: '#db2777', // deep rose pink
        accentColor: '#fb7185', // soft coral rose
        bgPreview: '#fff5f7'
    },
    {
        id: 'mint-matcha',
        name: 'Mint Matcha',
        description: 'Organic calming green tea light mode',
        primaryColor: '#0f766e', // deep matcha green
        accentColor: '#d97706', // warm golden honey
        bgPreview: '#f0fdf4'
    }
];

function createThemeStore() {
    let currentTheme = $state('slate-dark');

    return {
        get currentTheme() {
            return currentTheme;
        },
        get palettes() {
            return THEME_PALETTES;
        },
        init() {
            if (typeof window !== 'undefined') {
                const saved = localStorage.getItem('nomi-theme');
                if (saved && THEME_PALETTES.some(p => p.id === saved)) {
                    currentTheme = saved;
                }
                this.applyTheme(currentTheme);
            }
        },
        setTheme(themeId: string) {
            if (THEME_PALETTES.some(p => p.id === themeId)) {
                currentTheme = themeId;
                if (typeof window !== 'undefined') {
                    localStorage.setItem('nomi-theme', themeId);
                }
                this.applyTheme(themeId);
            }
        },
        applyTheme(themeId: string) {
            if (typeof document !== 'undefined') {
                const root = document.documentElement;
                root.setAttribute('data-theme', themeId);
                
                // Keep body background matching to prevent flash/borders
                const selected = THEME_PALETTES.find(p => p.id === themeId);
                if (selected) {
                    document.body.style.backgroundColor = selected.bgPreview;
                }
            }
        }
    };
}

export const themeStore = createThemeStore();
