import type { AuthData } from '$lib/generated/persepolis/AuthData';
import { writable, type Writable } from 'svelte/store';

export interface OBBoundary {
	authData: AuthData;
	token: string;
}

export const obBoundary: Writable<OBBoundary | null> = writable(null);
