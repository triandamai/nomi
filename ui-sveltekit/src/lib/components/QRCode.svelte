<script lang="ts">
    import qrcode from 'qrcode-generator';
    import { onMount } from 'svelte';

    let { data = '', size = 256, level = 'L' } = $props<{
        data?: string;
        size?: number;
        level?: 'L' | 'M' | 'Q' | 'H';
    }>();

    let qrContainer = $state<HTMLDivElement | null>(null);

    onMount(() => {
        if (data) {
            const qr = qrcode(0, level);
            qr.addData(data);
            qr.make();
            if (qrContainer) {
                qrContainer.innerHTML = qr.createSvgTag(Math.floor(size / qr.getModuleCount()), 0);
            }
        }
    });

    $effect(() => {
        if (data && qrContainer) {
            const qr = qrcode(0, level);
            qr.addData(data);
            qr.make();
            qrContainer.innerHTML = qr.createSvgTag(Math.floor(size / qr.getModuleCount()), 0);
        }
    });
</script>

<div bind:this={qrContainer} class="bg-white p-2 rounded-lg flex items-center justify-center overflow-hidden" style="width: {size}px; height: {size}px;">
    {#if !data}
        <div class="flex flex-col items-center gap-2">
            <div class="w-8 h-8 border-4 border-zinc-200 border-t-emerald-500 rounded-full animate-spin"></div>
            <p class="text-[10px] text-zinc-400 font-bold uppercase tracking-widest">Generating...</p>
        </div>
    {/if}
</div>
