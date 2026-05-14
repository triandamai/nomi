<script lang="ts">
	import { popupStore } from '$lib/stores/popup.svelte';
	import { Zap, Check, Shield, Cpu, MessageSquare, Database, Sparkles } from 'lucide-svelte';

	const PRICING_PLANS = [
		{
			name: 'Nomi',
			tagline: 'The Reliable Partner',
			price: 'Free',
			tokens: '700K',
			description: 'Perfect for getting started and organizing your daily multimodality.',
			color: 'slate',
			icon: MessageSquare,
			features: ['Finance Snap & Log', 'Vitality Live Sync', ' pgvector Memory']
		},
		{
			name: 'Nomi Max',
			tagline: 'Infinite Context',
			price: '$12/mo',
			tokens: '10M+',
			description: 'For power users who need deep reasoning and massive memory context.',
			color: 'blue',
			icon: Sparkles,
			features: ['All Nomi Features', 'Unlimited Priority Processing', 'Advanced RAG Search'],
			popular: true
		},
		{
			name: 'Nomi Decode',
			tagline: 'Hyper Scalable',
			price: 'PAYG',
			tokens: '∞',
			description: 'Professional grade pay-as-you-go. Pay only for the brainpower you consume.',
			color: 'emerald',
			icon: Zap,
			features: ['Dynamic Token Scaling', 'Early Access Features', 'API Access (Coming Soon)']
		}
	];
</script>

<div class="flex flex-col h-full text-slate-200">
	<div class="p-6 text-center space-y-4">
		<div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-rose-500/10 border border-rose-500/20 mb-2">
			<Shield class="w-3.5 h-3.5 text-rose-500" />
			<span class="text-[10px] font-black uppercase tracking-widest text-rose-500">Resource Limit Reached</span>
		</div>
		<h2 class="text-2xl font-black text-white tracking-tight">Upgrade Your Brainpower</h2>
		<p class="text-sm text-slate-400 max-w-sm mx-auto leading-relaxed">
			Nomi's current context is full. Upgrade to expand your memory and keep the conversation flowing.
		</p>
	</div>

	<div class="flex-1 overflow-y-auto p-4 space-y-4 custom-scrollbar">
		{#each PRICING_PLANS as plan}
			<div class="relative group">
				{#if plan.popular}
					<div class="absolute -top-3 left-1/2 -translate-x-1/2 z-10">
						<span class="bg-blue-600 text-[9px] font-black uppercase tracking-widest px-3 py-1 rounded-full text-white shadow-lg shadow-blue-900/40">
							Most Popular
						</span>
					</div>
				{/if}
				
				<button class="w-full text-left bg-slate-900/50 border {plan.popular ? 'border-blue-500/50 shadow-[0_0_30px_-10px_rgba(59,130,246,0.3)]' : 'border-slate-800'} rounded-3xl p-6 transition-all hover:bg-slate-900 group active:scale-[0.98]">
					<div class="flex items-start justify-between mb-4">
						<div class="flex items-center gap-4">
							<div class="w-12 h-12 rounded-2xl flex items-center justify-center transition-transform group-hover:scale-110 {plan.color === 'blue' ? 'bg-blue-500/10 text-blue-400' : plan.color === 'emerald' ? 'bg-emerald-500/10 text-emerald-400' : 'bg-slate-800 text-slate-400'}">
								<plan.icon size={24} />
							</div>
							<div>
								<h3 class="font-black text-white uppercase tracking-tighter text-lg">{plan.name}</h3>
								<p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest">{plan.tagline}</p>
							</div>
						</div>
						<div class="text-right">
							<p class="text-xl font-black text-white leading-none">{plan.price}</p>
							<p class="text-[10px] font-bold text-slate-500 uppercase tracking-widest mt-1">{plan.tokens} Tokens</p>
						</div>
					</div>

					<p class="text-xs text-slate-400 mb-6 leading-relaxed">
						{plan.description}
					</p>

					<div class="grid grid-cols-2 gap-y-2">
						{#each plan.features as feature}
							<div class="flex items-center gap-2">
								<Check class="w-3 h-3 {plan.color === 'blue' ? 'text-blue-500' : plan.color === 'emerald' ? 'text-emerald-500' : 'text-slate-500'}" />
								<span class="text-[10px] font-medium text-slate-400">{feature}</span>
							</div>
						{/each}
					</div>
				</button>
			</div>
		{/each}
	</div>

	<div class="p-6 border-t border-slate-800 flex justify-center">
		<button 
			onclick={() => popupStore.closeLast()}
			class="text-[10px] font-black uppercase tracking-widest text-slate-500 hover:text-slate-200 transition-colors"
		>
			I'll do it later
		</button>
	</div>
</div>

<style>
	.custom-scrollbar::-webkit-scrollbar {
		width: 4px;
	}
	.custom-scrollbar::-webkit-scrollbar-track {
		background: transparent;
	}
	.custom-scrollbar::-webkit-scrollbar-thumb {
		background: #1e293b;
		border-radius: 10px;
	}
	.custom-scrollbar::-webkit-scrollbar-thumb:hover {
		background: #334155;
	}
</style>
