<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { auth, isLoggedIn } from '$lib/stores/auth';
	import { notes, type Note } from '$lib/stores/notes';
	import { goto } from '$app/navigation';

	let selectedNote = $state<Note | null>(null);
	let title = $state('');
	let content = $state('');
	let syncing = $state(false);
	let syncMsg = $state('');
	let backendErr = $state('');

	onMount(() => {
		const unsub = isLoggedIn.subscribe(($in) => {
			if (!$in) goto('/login');
		});
		loadNotes();
		return unsub;
	});

	async function loadNotes() {
		try {
			const result: any = await invoke('get_notes');
			notes.set(result as Note[]);
		} catch (err: any) {
			backendErr = typeof err === 'string' ? err : 'Failed to load notes';
			setTimeout(() => (backendErr = ''), 4000);
		}
	}

	async function selectNote(note: Note) {
		selectedNote = note;
		title = note.title;
		content = note.content;
	}

	function newNote() {
		selectedNote = null;
		title = '';
		content = '';
	}

	async function saveNote() {
		if (!title.trim() && !content.trim()) return;
		try {
			backendErr = '';
			if (selectedNote) {
				const updated: any = await invoke('update_note', {
					id: selectedNote.id,
					title,
					content,
				});
				notes.updateNote(updated as Note);
				selectedNote = updated as Note;
			} else {
				const created: any = await invoke('create_note', { title, content });
				notes.update((ns) => [created as Note, ...ns]);
				selectedNote = created as Note;
			}
		} catch (err: any) {
			backendErr = typeof err === 'string' ? err : 'Save failed — backend maybe offline';
			setTimeout(() => (backendErr = ''), 4000);
		}
	}

	async function deleteCurrentNote() {
		if (!selectedNote) return;
		try {
			backendErr = '';
			await invoke('delete_note', { id: selectedNote.id });
			notes.removeNote(selectedNote.id);
			selectedNote = null;
			title = '';
			content = '';
		} catch (err: any) {
			backendErr = typeof err === 'string' ? err : 'Delete failed — backend maybe offline';
			setTimeout(() => (backendErr = ''), 4000);
		}
	}

	async function handleSync() {
		syncing = true;
		syncMsg = 'Syncing...';
		try {
			const result: any = await invoke('sync_notes');
			notes.set(result as Note[]);
			syncMsg = 'Synced ✓';
		} catch (err: any) {
			syncMsg = 'Sync failed';
			console.error('Sync failed', err);
		} finally {
			syncing = false;
			setTimeout(() => (syncMsg = ''), 3000);
		}
	}

	function handleLogout() {
		auth.logout();
		goto('/login');
	}

	$effect(() => {
		// Auto-save when switching notes
	});
</script>

{#if backendErr}<p class="backend-error">{backendErr}</p>{/if}

<div class="app-layout">
	<aside class="sidebar">
		<div class="sidebar-header">
			<h2>Notes</h2>
			<div class="sidebar-actions">
				<button class="icon-btn" onclick={newNote} title="New note">+</button>
				<button class="icon-btn" onclick={handleSync} disabled={syncing} title="Sync">
					&#x21bb;
				</button>
				<button class="icon-btn logout" onclick={handleLogout} title="Logout">&rarr;</button>
			</div>
		</div>
		{#if syncMsg}<p class="sync-msg">{syncMsg}</p>{/if}
		<div class="note-list">
			{#each $notes as note (note.id)}
				<button
					class="note-card"
					class:selected={selectedNote?.id === note.id}
					onclick={() => selectNote(note)}
				>
					<strong>{note.title || 'Untitled'}</strong>
					<span class="preview">{note.content.slice(0, 60)}</span>
					{#if note.sync_status !== 'synced'}
						<span class="unsynced" title="Not synced">&#x25cf;</span>
					{/if}
				</button>
			{/each}
		</div>
	</aside>

	<main class="editor">
		{#if selectedNote || !title}
			<input
				class="title-input"
				placeholder="Note title..."
				bind:value={title}
				oninput={saveNote}
			/>
			<textarea
				class="content-input"
				placeholder="Start writing..."
				bind:value={content}
				oninput={saveNote}
			></textarea>
			<div class="editor-footer">
				<button class="delete-btn" onclick={deleteCurrentNote}>Delete</button>
			</div>
		{:else}
			<div class="empty-state">
				<p>Select a note or create a new one</p>
			</div>
		{/if}
	</main>
</div>

<style>
	.app-layout { display: flex; height: 100vh; }
	.sidebar { width: 280px; border-right: 1px solid #ddd; display: flex; flex-direction: column; background: #f9f9f9; }
	.sidebar-header { padding: 1rem; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #ddd; }
	.sidebar-header h2 { margin: 0; font-size: 1.1rem; }
	.sidebar-actions { display: flex; gap: 0.25rem; }
	.icon-btn { width: 32px; height: 32px; border: 1px solid #ccc; border-radius: 6px; background: #fff; cursor: pointer; font-size: 1rem; display: flex; align-items: center; justify-content: center; }
	.icon-btn.logout { font-size: 1.2rem; }
	.sync-msg { padding: 0.25rem 1rem; font-size: 0.75rem; color: #666; }
	.note-list { flex: 1; overflow-y: auto; padding: 0.5rem; }
	.note-card { display: block; width: 100%; text-align: left; padding: 0.75rem; margin-bottom: 0.25rem; border: 1px solid transparent; border-radius: 6px; background: transparent; cursor: pointer; }
	.note-card.selected { background: #e0e7ff; border-color: #396cd8; }
	.note-card strong { display: block; font-size: 0.9rem; }
	.preview { font-size: 0.8rem; color: #888; display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.unsynced { color: #f59e0b; font-size: 0.75rem; }
	.editor { flex: 1; display: flex; flex-direction: column; padding: 1.5rem; }
	.title-input { font-size: 1.5rem; font-weight: 600; border: none; outline: none; padding: 0.5rem 0; margin-bottom: 0.5rem; width: 100%; }
	.content-input { flex: 1; border: none; outline: none; resize: none; font-size: 1rem; line-height: 1.6; width: 100%; }
	.editor-footer { padding-top: 0.5rem; border-top: 1px solid #eee; }
	.delete-btn { padding: 0.4rem 1rem; border: 1px solid #e33; border-radius: 6px; background: #fff; color: #e33; cursor: pointer; }
	.empty-state { display: flex; align-items: center; justify-content: center; height: 100%; color: #aaa; }
	.backend-error { margin: 0; padding: 0.5rem 1rem; background: #fee2e2; color: #b91c1c; font-size: 0.8rem; text-align: center; }
</style>
