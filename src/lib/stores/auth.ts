import { writable, derived } from 'svelte/store';

export interface User {
	id: string;
	username: string;
}

export interface AuthState {
	user: User | null;
	token: string | null;
}

function createAuth() {
	const stored = typeof localStorage !== 'undefined' ? localStorage.getItem('auth') : null;
	const initial: AuthState = stored ? JSON.parse(stored) : { user: null, token: null };
	const { subscribe, set } = writable<AuthState>(initial);

	return {
		subscribe,
		login(user: User, token: string) {
			set({ user, token });
			localStorage.setItem('auth', JSON.stringify({ user, token }));
		},
		logout() {
			set({ user: null, token: null });
			localStorage.removeItem('auth');
		},
	};
}

export const auth = createAuth();
export const isLoggedIn = derived(auth, ($a) => $a.user !== null && $a.token !== null);
export const currentUser = derived(auth, ($a) => $a.user);
