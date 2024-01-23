import { writable, type Writable } from 'svelte/store';
import type { CdnAssetItem } from '$lib/generated/arcadia/CdnAssetItem';

export interface CdnStateStore {
	path: string;
	triggerRefresh: number;
}

export interface CdnDataStore {
	files: CdnAssetItem[];
}

export const cdnStateStore: Writable<CdnStateStore> = writable({
	path: '',
	triggerRefresh: 0
});

export const cdnDataStore: Writable<CdnDataStore> = writable({
	files: []
});
