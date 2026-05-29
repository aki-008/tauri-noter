<script lang="ts">
	import { onMount } from 'svelte';
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';

	onMount(() => {
		const unsub = auth.subscribe(($auth) => {
			if ($auth.user && $auth.token) {
				goto('/notes');
			} else {
				goto('/login');
			}
		});
		return unsub;
	});
</script>
