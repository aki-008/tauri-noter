<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	async function handleLogin(e: Event) {
		e.preventDefault();
		error = '';
		loading = true;
		try {
			const resp: any = await invoke('login', { username, password });
			auth.login(resp.user, resp.access_token);
			goto('/notes');
		} catch (err: any) {
			error = typeof err === 'string' ? err : 'Login failed';
		} finally {
			loading = false;
		}
	}
</script>

<main class="auth-page">
	<h1>Note Taker</h1>
	<form onsubmit={handleLogin}>
		<h2>Login</h2>
		{#if error}<p class="error">{error}</p>{/if}
		<input type="text" placeholder="Username" bind:value={username} required />
		<input type="password" placeholder="Password" bind:value={password} required />
		<button type="submit" disabled={loading}>{loading ? 'Logging in...' : 'Login'}</button>
		<p class="link">
			Don't have an account? <a href="/register">Register</a>
		</p>
	</form>
</main>

<style>
	.auth-page { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; }
	form { display: flex; flex-direction: column; gap: 0.75rem; width: 300px; }
	input, button { padding: 0.6rem 1rem; font-size: 1rem; border-radius: 6px; border: 1px solid #ccc; }
	button { background: #396cd8; color: #fff; border: none; cursor: pointer; }
	button:disabled { opacity: 0.6; }
	.error { color: #e33; font-size: 0.875rem; }
	.link { font-size: 0.875rem; text-align: center; }
</style>
