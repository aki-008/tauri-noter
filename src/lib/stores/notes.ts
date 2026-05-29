import { writable } from 'svelte/store';

export interface Note {
	id: string;
	title: string;
	content: string;
	created_at: string;
	updated_at: string;
	sync_status: string;
}

function createNotes() {
	const { subscribe, set, update } = writable<Note[]>([]);

	return {
		subscribe,
		set,
		update,
		reset() {
			set([]);
		},
		updateNote(updated: Note) {
			update((ns) => ns.map((n) => (n.id === updated.id ? updated : n)));
		},
		removeNote(id: string) {
			update((ns) => ns.filter((n) => n.id !== id));
		},
	};
}

export const notes = createNotes();
