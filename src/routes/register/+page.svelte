<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';

	let username = $state('');
	let password = $state('');
	let confirm = $state('');
	let error = $state('');
	let loading = $state(false);

	async function handleRegister(e: Event) {
		e.preventDefault();
		error = '';
		if (password !== confirm) {
			error = 'Passwords do not match';
			return;
		}
		loading = true;
		try {
			const resp: any = await invoke('register', { username, password });
			auth.login(resp.user, resp.access_token);
			goto('/notes');
		} catch (err: any) {
			error = typeof err === 'string' ? err : 'Registration failed';
		} finally {
			loading = false;
		}
	}
</script>

<main class="auth-page">
	<h1>Note Taker</h1>
	<form onsubmit={handleRegister}>
		<h2>Register</h2>
		{#if error}<p class="error">{error}</p>{/if}
		<input type="text" placeholder="Username" bind:value={username} required minlength={3} />
		<input type="password" placeholder="Password" bind:value={password} required minlength={4} />
		<input type="password" placeholder="Confirm Password" bind:value={confirm} required />
		<button type="submit" disabled={loading}>{loading ? 'Creating account...' : 'Register'}</button>
		<p class="link">
			Already have an account? <a href="/login">Login</a>
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
