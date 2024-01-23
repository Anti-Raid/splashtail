import { writable, type Writable } from 'svelte/store';
import type { Hello } from './generated/arcadia/Hello';

export const panelState: Writable<Hello | null> = writable(null);
