import { get } from 'svelte/store';
import type { PanelQuery } from '$lib/generated/arcadia/PanelQuery';
import logger from './logger';
import { logoutUser } from './logout';
import { panelAuthState } from './panelAuthState';

export const panelQuery = async (query: PanelQuery) => {
	let data = get(panelAuthState);

	return await fetchClient(`${data?.url}`, {
		headers: {
			'Content-Type': 'application/json'
		},
		method: 'POST',
		body: JSON.stringify(query)
	});
};

export const fetchClient = async (url: string, init?: RequestInit) => {
	logger.info('FetchClient', init?.method || 'GET', url);

	const response = await fetch(url, init);

	if (!response.ok) {
		if (response.status == 408) {
			throw new Error('Server down for maintenance');
		}

		// Open up the response body using clone()
		const body = await response.clone().text();

		if (body == 'identityExpired') {
			logoutUser(false);
			throw new Error('Session expired...');
		}
	}

	return response;
};
