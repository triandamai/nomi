<script lang="ts">
    import { onMount } from 'svelte';
    import { chatApi } from '$lib/api/client';
    import { popupStore } from '$lib/stores/popup.svelte';
    import { Search, Edit2, Trash2, Check, X, ChevronLeft, ChevronRight, Loader2, List } from 'lucide-svelte';

    let transactions = $state<any[]>([]);
    let totalCount = $state(0);
    let currentPage = $state(1);
    let searchQuery = $state('');
    let selectedCategory = $state<string | undefined>(undefined);
    let isLoading = $state(false);

    let searchTimeout: ReturnType<typeof setTimeout>;

    const CATEGORIES = [
        { icon: '🍔', name: 'Food' },
        { icon: '⛽', name: 'Fuel' },
        { icon: '🛒', name: 'Shopping' },
        { icon: '🏔️', name: 'Leisure' }
    ];

    let editingId = $state<string | null>(null);
    let editAmount = $state<number>(0);
    let editMerchant = $state('');
    let editCategory = $state('');
    
    let selectedDetailTransaction = $state<any>(null);

    async function fetchTransactions() {
        isLoading = true;
        try {
            const res = await chatApi.getMoneyHistory(currentPage, searchQuery, selectedCategory);
            transactions = res.data.items;
            totalCount = res.data.total_count;
        } catch (e) {
            console.error(e);
        } finally {
            isLoading = false;
        }
    }

    onMount(() => {
        fetchTransactions();
    });

    function handleSearch(e: Event) {
        const val = (e.target as HTMLInputElement).value;
        searchQuery = val;
        currentPage = 1;
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            fetchTransactions();
        }, 300);
    }

    function toggleCategory(cat: string) {
        if (selectedCategory === cat) {
            selectedCategory = undefined;
        } else {
            selectedCategory = cat;
        }
        currentPage = 1;
        fetchTransactions();
    }

    function startEdit(t: any) {
        editingId = t.id;
        editAmount = parseFloat(t.total_amount);
        editMerchant = t.merchant_name || '';
        editCategory = t.category || '';
    }

    function cancelEdit() {
        editingId = null;
    }

    async function saveEdit(id: string) {
        try {
            await chatApi.updateMoneyHistory(id, {
                amount: editAmount,
                merchant_name: editMerchant,
                category: editCategory
            });
            // Update local state
            const idx = transactions.findIndex(t => t.id === id);
            if (idx !== -1) {
                transactions[idx] = {
                    ...transactions[idx],
                    total_amount: editAmount.toString(),
                    merchant_name: editMerchant,
                    category: editCategory
                };
            }
            editingId = null;
        } catch (e) {
            console.error(e);
        }
    }

    async function deleteTransaction(id: string) {
        // Optimistic update
        const previousTransactions = [...transactions];
        transactions = transactions.filter(t => t.id !== id);
        totalCount -= 1;

        try {
            await chatApi.deleteMoneyHistory(id);
        } catch (e) {
            console.error(e);
            // Revert on failure
            transactions = previousTransactions;
            totalCount += 1;
        }
    }

    function prevPage() {
        if (currentPage > 1) {
            currentPage--;
            fetchTransactions();
        }
    }

    function nextPage() {
        if (currentPage * 20 < totalCount) {
            currentPage++;
            fetchTransactions();
        }
    }
    
    function showDetails(t: any) {
        selectedDetailTransaction = t;
        popupStore.open({
            title: 'Transaction Details',
            width: 'max-w-md',
            contentSnippet: detailSnippet
        });
    }
</script>

{#snippet detailSnippet()}
    {#if selectedDetailTransaction}
        <div class="space-y-6 py-2">
            <div class="flex justify-between items-start gap-4">
                <div>
                    <h3 class="font-black text-xl text-slate-100">{selectedDetailTransaction.merchant_name || 'Unknown Merchant'}</h3>
                    {#if selectedDetailTransaction.category}
                        <span class="inline-block mt-1 px-2.5 py-1 text-[10px] rounded-full bg-slate-800 text-slate-300 uppercase font-black tracking-widest border border-slate-700">{selectedDetailTransaction.category}</span>
                    {/if}
                </div>
                <div class="text-right shrink-0">
                    <p class="font-bold text-rose-400 text-xl font-mono">
                        {new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR' }).format(parseFloat(selectedDetailTransaction.total_amount))}
                    </p>
                    <p class="text-[11px] text-slate-500 font-mono mt-1">{new Date(selectedDetailTransaction.created_at).toLocaleString()}</p>
                </div>
            </div>
            {#if selectedDetailTransaction.description}
                <div class="bg-slate-900/50 border border-slate-800/50 p-4 rounded-2xl">
                    <p class="text-sm text-slate-300 leading-relaxed">{selectedDetailTransaction.description}</p>
                </div>
            {/if}
            
            <div>
                <div class="flex items-center gap-2 mb-3">
                    <List class="w-4 h-4 text-slate-500" />
                    <h4 class="text-[10px] font-black uppercase tracking-widest text-slate-500">Purchased Items</h4>
                </div>
                {#if selectedDetailTransaction.items && selectedDetailTransaction.items.length > 0}
                    <div class="space-y-2">
                        {#each selectedDetailTransaction.items as item}
                            <div class="flex justify-between items-center bg-slate-950 border border-slate-800 p-4 rounded-2xl hover:border-slate-700 transition-colors">
                                <div class="flex-1 min-w-0 pr-4">
                                    <p class="text-sm font-bold text-slate-200 truncate">{item.name}</p>
                                    <p class="text-[11px] text-slate-500 mt-1 font-mono uppercase tracking-widest">Qty: {item.quantity}</p>
                                </div>
                                <p class="text-sm font-bold text-slate-300 shrink-0 font-mono">
                                    {new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR' }).format(parseFloat(item.total_amount))}
                                </p>
                            </div>
                        {/each}
                    </div>
                {:else}
                    <div class="text-center py-6 bg-slate-900/30 rounded-2xl border border-slate-800/30">
                        <p class="text-sm text-slate-500 italic">No specific items recorded.</p>
                    </div>
                {/if}
            </div>
        </div>
    {/if}
{/snippet}

<div class="flex flex-col h-full text-slate-200">
    <div class="p-4 border-b border-slate-800 space-y-4 bg-[#0f172a]/50 backdrop-blur-md">
        <!-- Search Bar -->
        <div class="relative">
            <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
            <input 
                type="text" 
                placeholder="Search merchant or description..." 
                oninput={handleSearch}
                value={searchQuery}
                class="w-full bg-slate-900 border border-slate-800 rounded-2xl pl-11 pr-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 transition-all placeholder:text-slate-600"
            />
        </div>

        <!-- Category Pills -->
        <div class="flex flex-wrap gap-2">
            {#each CATEGORIES as cat}
                <button 
                    onclick={() => toggleCategory(cat.name)}
                    class="px-4 py-1.5 rounded-full text-xs font-bold flex items-center gap-2 transition-all
                           {selectedCategory === cat.name ? 'bg-blue-500/20 text-blue-400 border border-blue-500/30' : 'bg-slate-900 text-slate-400 border border-slate-800 hover:bg-slate-800'}"
                >
                    <span>{cat.icon}</span>
                    <span>{cat.name}</span>
                </button>
            {/each}
        </div>
    </div>

    <!-- Transactions List -->
    <div class="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar">
        {#if isLoading && transactions.length === 0}
            <div class="flex justify-center py-12">
                <Loader2 class="w-8 h-8 animate-spin text-blue-500" />
            </div>
        {:else if transactions.length === 0}
            <div class="text-center py-12 text-sm text-slate-500">
                No transactions found.
            </div>
        {:else}
            {#each transactions as t (t.id)}
                <div class="bg-slate-900/50 border border-slate-800/50 rounded-2xl p-4 group hover:border-slate-700 transition-colors">
                    {#if editingId === t.id}
                        <div class="space-y-4">
                            <div class="grid grid-cols-2 gap-3">
                                <div>
                                    <label class="text-[10px] uppercase text-slate-500 font-black tracking-widest mb-1 block">Amount</label>
                                    <input type="number" bind:value={editAmount} class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50 font-mono" />
                                </div>
                                <div>
                                    <label class="text-[10px] uppercase text-slate-500 font-black tracking-widest mb-1 block">Category</label>
                                    <input type="text" bind:value={editCategory} class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50" />
                                </div>
                                <div class="col-span-2">
                                    <label class="text-[10px] uppercase text-slate-500 font-black tracking-widest mb-1 block">Merchant</label>
                                    <input type="text" bind:value={editMerchant} class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500/50" />
                                </div>
                            </div>
                            <div class="flex justify-end gap-2">
                                <button onclick={cancelEdit} class="px-4 py-2 text-xs font-bold text-slate-400 hover:text-slate-200 bg-slate-800 hover:bg-slate-700 rounded-xl transition-colors">Cancel</button>
                                <button onclick={() => saveEdit(t.id)} class="px-4 py-2 text-xs font-bold text-white bg-blue-600 hover:bg-blue-500 rounded-xl transition-colors flex items-center gap-1.5"><Check class="w-4 h-4" /> Save</button>
                            </div>
                        </div>
                    {:else}
                        <div class="flex items-start justify-between gap-4">
                            <div class="flex-1 min-w-0" role="button" tabindex="0" onclick={() => showDetails(t)} onkeydown={(e) => e.key === 'Enter' && showDetails(t)}>
                                <div class="flex items-center gap-2 mb-1.5">
                                    <span class="font-bold text-slate-200 truncate">{t.merchant_name || 'Unknown Merchant'}</span>
                                    {#if t.category}
                                        <span class="px-2 py-0.5 text-[9px] rounded-full bg-slate-800 text-slate-400 uppercase font-black tracking-widest border border-slate-700">{t.category}</span>
                                    {/if}
                                </div>
                                <p class="text-xs text-slate-500 truncate mb-1.5">{t.description || 'No description'}</p>
                                <p class="text-[10px] text-slate-600 font-mono tracking-wider uppercase">{new Date(t.created_at).toLocaleString()}</p>
                            </div>
                            
                            <div class="flex flex-col items-end gap-2 shrink-0">
                                <div class="font-bold text-rose-400 font-mono text-sm">
                                    {new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR' }).format(parseFloat(t.total_amount))}
                                </div>
                                <div class="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <button onclick={() => startEdit(t)} class="p-1.5 text-slate-500 hover:text-blue-400 bg-slate-800/50 hover:bg-slate-800 rounded-lg transition-colors" title="Edit"><Edit2 class="w-3.5 h-3.5" /></button>
                                    <button onclick={() => deleteTransaction(t.id)} class="p-1.5 text-slate-500 hover:text-rose-400 bg-slate-800/50 hover:bg-slate-800 rounded-lg transition-colors" title="Delete"><Trash2 class="w-3.5 h-3.5" /></button>
                                </div>
                            </div>
                        </div>
                    {/if}
                </div>
            {/each}
        {/if}
    </div>

    <!-- Pagination Footer -->
    {#if totalCount > 20}
        <div class="p-4 border-t border-slate-800 flex items-center justify-between bg-slate-950">
            <span class="text-[11px] font-bold text-slate-500 uppercase tracking-widest">
                Showing <span class="font-mono text-slate-300">{(currentPage - 1) * 20 + 1}</span> - <span class="font-mono text-slate-300">{Math.min(currentPage * 20, totalCount)}</span> of <span class="font-mono text-slate-300">{totalCount}</span>
            </span>
            <div class="flex items-center gap-2">
                <button 
                    onclick={prevPage}
                    disabled={currentPage === 1}
                    class="p-2 rounded-xl bg-slate-900 border border-slate-800 text-slate-400 hover:bg-slate-800 hover:text-slate-200 disabled:opacity-50 disabled:pointer-events-none transition-colors"
                >
                    <ChevronLeft class="w-4 h-4" />
                </button>
                <button 
                    onclick={nextPage}
                    disabled={currentPage * 20 >= totalCount}
                    class="p-2 rounded-xl bg-slate-900 border border-slate-800 text-slate-400 hover:bg-slate-800 hover:text-slate-200 disabled:opacity-50 disabled:pointer-events-none transition-colors"
                >
                    <ChevronRight class="w-4 h-4" />
                </button>
            </div>
        </div>
    {/if}
</div>
