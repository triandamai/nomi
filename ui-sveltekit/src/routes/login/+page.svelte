<script lang="ts">
    import { chatApi } from '$lib/api/client';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { page } from '$app/state';
    import { Zap, ShieldCheck, ArrowRight, Loader2, Lock } from 'lucide-svelte';
    import { fly, fade } from 'svelte/transition';

    let externalId = $state('');
    let channel = $state('device');
    let code = $state('');
    let loading = $state(false);
    let message = $state('');
    let error = $state('');

    onMount(() => {
        const id = page.url.searchParams.get('id');
        if (id) {
            externalId = id;
            message = 'A verification code has been sent to your account.';
        }
    });

    async function handleVerifyOtp(e: Event) {
        e.preventDefault();
        if (code.length < 6) return;
        
        loading = true;
        error = '';
        try {
            const response = await chatApi.verifyOtp(externalId, code);
            sessionStorage.setItem('auth_token', response.data.access_token);
            sessionStorage.setItem('user_id', response.data.user_id);
            goto('/chat');
        } catch (e: any) {
            error = e.message || 'Invalid or expired OTP';
        } finally {
            loading = false;
        }
    }
</script>

<div class="min-h-screen bg-[#0f172a] text-slate-200 font-sans selection:bg-blue-500/30 flex flex-col">
    <!-- Header/Logo -->
    <header class="p-8">
        <a href="/" class="inline-flex items-center gap-2 group">
            <div class="w-10 h-10 bg-blue-600 rounded-xl flex items-center justify-center shadow-lg shadow-blue-500/20 group-hover:scale-110 transition-transform">
                <Zap class="w-6 h-6 text-white fill-white" />
            </div>
            <span class="text-2xl font-black tracking-tighter text-white uppercase">NOMI</span>
        </a>
    </header>

    <main class="flex-1 flex items-center justify-center p-6 pb-24">
        <div class="w-full max-w-[440px] relative">
            <!-- Background Glow -->
            <div class="absolute -inset-20 bg-gradient-to-tr from-blue-500/10 to-emerald-500/10 blur-3xl rounded-full opacity-50"></div>

            <div class="relative space-y-8" in:fly={{ y: 20, duration: 800 }}>
                <!-- Title -->
                <div class="text-center space-y-2">
                    <h1 class="text-3xl font-black text-white tracking-tight">Login</h1>
                    <p class="text-slate-400 text-sm">Secure entry to your Nomi workspace</p>
                </div>

                <!-- Login Card -->
                <div class="bg-slate-900/50 border border-slate-800 rounded-[2rem] p-8 md:p-10 shadow-2xl backdrop-blur-xl">
                    <form onsubmit={handleVerifyOtp} class="space-y-8">
                        <div class="space-y-6 text-center">
                            <div class="inline-flex items-center justify-center w-16 h-16 bg-slate-800 rounded-2xl border border-slate-700 mb-2">
                                <Lock class="w-8 h-8 text-blue-400" />
                            </div>
                            
                            <div class="space-y-2">
                                <label for="otp" class="text-[10px] font-black uppercase tracking-[0.2em] text-slate-500">
                                    Verification Code
                                </label>
                                <p class="text-xs text-slate-400">Enter the 6-digit code sent to your {channel}</p>
                            </div>

                            <div class="relative">
                                <input
                                    id="otp"
                                    bind:value={code}
                                    type="text"
                                    placeholder="••••••"
                                    maxlength="6"
                                    autocomplete="one-time-code"
                                    class="w-full bg-slate-950 border border-slate-800 text-white text-4xl tracking-[0.4em] font-mono text-center rounded-2xl px-4 py-6 focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500/50 outline-none transition-all placeholder:text-slate-800"
                                    required
                                />
                            </div>
                        </div>

                        <button
                            type="submit"
                            disabled={loading || code.length < 6}
                            class="w-full h-14 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white font-bold rounded-2xl transition-all flex items-center justify-center gap-3 shadow-lg shadow-blue-500/20 group"
                        >
                            {#if loading}
                                <Loader2 class="w-5 h-5 animate-spin" />
                                Authenticating...
                            {:else}
                                Verify & Enter
                                <ArrowRight class="w-5 h-5 group-hover:translate-x-1 transition-transform" />
                            {/if}
                        </button>
                    </form>

                    {#if message || error}
                        <div class="mt-8 pt-8 border-t border-slate-800/50" in:fade>
                            {#if message}
                                <div class="flex items-center gap-3 px-4 py-3 bg-emerald-500/10 border border-emerald-500/20 rounded-xl">
                                    <ShieldCheck class="w-4 h-4 text-emerald-400 shrink-0" />
                                    <p class="text-emerald-400 text-xs font-medium">{message}</p>
                                </div>
                            {/if}
                            {#if error}
                                <div class="flex items-center gap-3 px-4 py-3 bg-rose-500/10 border border-rose-500/20 rounded-xl">
                                    <div class="w-1.5 h-1.5 rounded-full bg-rose-500 shrink-0"></div>
                                    <p class="text-rose-400 text-xs font-medium">{error}</p>
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>

                <!-- Footer Links -->
                <div class="text-center space-y-4">
                    <p class="text-[10px] text-slate-600 font-bold uppercase tracking-widest">
                        Protected by End-to-End Encryption
                    </p>
                    <div class="flex justify-center gap-6">
                        <a href="/" class="text-xs text-slate-500 hover:text-white transition-colors">Back to Home</a>
                        <span class="text-slate-800">|</span>
                        <a href="/privacy" class="text-xs text-slate-500 hover:text-white transition-colors">Security Protocol</a>
                    </div>
                </div>
            </div>
        </div>
    </main>

    <footer class="p-8 mt-auto border-t border-slate-800/30">
        <p class="text-center text-[10px] text-slate-600 font-bold uppercase tracking-[0.2em]">
            Experimental AI System — v2.0 Beta
        </p>
    </footer>
</div>

<style>
    :global(body) {
        background-color: #0f172a;
    }
</style>
