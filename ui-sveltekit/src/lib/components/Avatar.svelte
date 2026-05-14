<script lang="ts">
    import { useAvatar } from '$lib/utils';

    interface Props {
        name: string;
        active?: boolean;
        online?: boolean;
        size?: 'sm' | 'md' | 'lg';
        onClick?: () => void;
    }

    let { 
        name, 
        active = false, 
        online = false, 
        size = 'md',
        onClick = () => {} 
    }: Props = $props();

    let avatarUrl = $derived(useAvatar(name));

    const sizeClasses: Record<'sm' | 'md' | 'lg', string> = {
        sm: 'w-8 h-8',
        md: 'w-12 h-12',
        lg: 'w-14 h-14'
    };
</script>

<button 
    onclick={onClick}
    class="relative group flex items-center justify-center transition-all duration-200"
>
    <!-- Discord-like Active Indicator -->
    <div class="absolute -left-3 w-1 bg-white rounded-r-full transition-all duration-200 
        {active ? 'h-8' : 'h-2 scale-0 group-hover:scale-100 group-hover:h-5'}">
    </div>

    <!-- Avatar Container -->
    <div class="{sizeClasses[size]} rounded-[24px] group-hover:rounded-[16px] transition-all duration-200 overflow-hidden bg-slate-800
        {active ? 'rounded-[16px] ring-2 ring-blue-500 ring-offset-2 ring-offset-[#0f172a]' : ''}
        {online && !active ? 'ring-2 ring-blue-500/50' : ''}">
        <img src={avatarUrl} alt={name} class="w-full h-full object-cover" />
    </div>

    <!-- Tooltip -->
    <div class="absolute left-16 px-3 py-1 bg-slate-950 text-white text-xs font-bold rounded shadow-xl whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity z-50">        {name}
    </div>
</button>
