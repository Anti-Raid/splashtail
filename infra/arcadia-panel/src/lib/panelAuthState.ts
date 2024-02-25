import { writable, type Writable } from 'svelte/store';

export interface PanelAuthState {
	url: string;
	loginToken: string;
	sessionState: string;
	authErr?: string;
}

export const panelAuthState: Writable<PanelAuthState | null> = writable(null);
