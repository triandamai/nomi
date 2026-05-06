<script lang="ts">
    import { chatApi } from '$lib/api/client';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';

    import { page } from '$app/stores';

    let externalId = $state('');
    let channel = $state('email'); // Default to email
    let code = $state('');
    let loading = $state(false);
    let message = $state('');
    let error = $state('');

    onMount(() => {
        const id = $page.url.searchParams.get('id');
        if (id) {
            externalId = id;
            message = 'Please enter the verification code sent to your chat app.';
        }
    });

    async function handleVerifyOtp(e: Event) {
        e.preventDefault();
        loading = true;
        error = '';
        try {
            const response = await chatApi.verifyOtp(externalId, code);
            sessionStorage.setItem('auth_token', response.data.access_token);
            sessionStorage.setItem('user_id', response.data.user_id);
            goto('/');
        } catch (e: any) {
            error = e.message || 'Invalid OTP';
        } finally {
            loading = false;
        }
    }
</script>

<div class="min-h-screen flex items-center justify-center bg-[--bg-main] p-6 selection:bg-[--color-cash-green] selection:text-black">
    <div class="w-full max-w-[420px] space-y-12">
        <!-- Logo/Brand Section -->
        <div class="flex flex-col items-center">
            <div class="h-10 w-10 bg-black border border-[--border-main] rounded-lg flex items-center justify-center mb-8 shadow-sm">
                 <span class="text-[--color-cash-green] font-bold text-xl">O</span>
            </div>
            <h1 class="text-2xl font-medium tracking-tight text-[--text-main]">
                Sign in to Open Agent
            </h1>
        </div>

        <div class="bg-[--bg-card] border border-[--border-main] rounded-xl p-8 shadow-2xl">
                <form onsubmit={handleVerifyOtp} class="space-y-6">
                    <div class="space-y-4 text-center">
                        <label for="otp" class="text-[11px] font-semibold uppercase tracking-[0.1em] text-[--text-muted]">
                            Verification Code
                        </label>
                        <p class="text-xs text-zinc-500">A security code has been sent to your {channel}</p>
                        <input
                            id="otp"
                            bind:value={code}
                            type="text"
                            placeholder="000000"
                            maxlength="6"
                            class="w-full bg-[--bg-main] border border-[--border-main] text-[--text-main] text-3xl tracking-[0.6em] font-mono text-center rounded-md px-4 py-4 focus:outline-none focus:ring-1 focus:ring-zinc-600 focus:border-zinc-500 transition-all"
                            required
                        />
                    </div>

                    <button
                        type="submit"
                        disabled={loading}
                        class="w-full bg-[--color-cash-green] bg-cash-green hover:bg-[#00e63c] disabled:opacity-50 text-black font-semibold text-sm py-2.5 rounded-md transition-all active:scale-[0.99] shadow-md"
                    >
                        {loading ? 'Authenticating...' : 'Verify Code'}
                    </button>
                </form>


            {#if message || error}
                <div class="mt-8 pt-6 border-t border-[--border-main]">
                    {#if message}
                        <div class="bg-emerald-500/10 border border-emerald-500/20 rounded-md p-3">
                            <p class="text-[--color-cash-green] text-center text-[11px] font-medium">{message}</p>
                        </div>
                    {/if}
                    {#if error}
                        <div class="bg-red-500/10 border border-red-500/20 rounded-md p-3">
                            <p class="text-red-400 text-center text-[11px] font-medium">{error}</p>
                        </div>
                    {/if}
                </div>
            {/if}
        </div>

        <p class="text-center text-[11px] text-zinc-600 leading-relaxed">
            By continuing, you agree to our 
            <a href="/terms" class="text-zinc-400 hover:text-white underline underline-offset-4">Terms of Service</a> and 
            <a href="/privacy" class="text-zinc-400 hover:text-white underline underline-offset-4">Privacy Policy</a>.
        </p>
    </div>
</div>
