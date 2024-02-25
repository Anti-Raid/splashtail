<script lang="ts">
	import { panelQuery } from '$lib/fetch';
	import type { AppListResponse } from '$lib/comp_types/apps';
	import { panelAuthState } from '$lib/panelAuthState';
	import ErrorComponent from '../../../components/Error.svelte';
	import Loading from '../../../components/Loading.svelte';
	import AppCard from './AppCard.svelte';
	import Select from '../../../components/inputs/select/Select.svelte';
	import { title } from '$lib/strings';

	let appPositionFilter: string = 'staff';
	let appStateFilter: string = 'pending';

	const fetchApps = async () => {
		let res = await panelQuery({
			PopplioStaff: {
				login_token: $panelAuthState?.loginToken || '',
				path: '/staff/apps',
				method: 'GET',
				body: ''
			}
		});

		if (!res.ok) {
			throw new Error('Failed to fetch apps');
		}

		let appResp: AppListResponse = await res.json();

		let positions: string[] = [];

		for (let app of appResp?.apps) {
			if (!positions.includes(app?.position)) {
				positions.push(app?.position);
			}
		}

		let states: string[] = [];

		for (let app of appResp?.apps) {
			if (!states.includes(app?.state)) {
				states.push(app?.state);
			}
		}

		return {
			appResp,
			positions,
			states
		};
	};
</script>

{#await fetchApps()}
	<Loading msg={'Loading apps...'} />
{:then appResp}
	<Select
		id="appPositionFilter"
		label="Filter by position"
		bind:value={appPositionFilter}
		choices={appResp?.positions?.map((pos) => ({ id: pos, value: pos, label: title(pos) }))}
	/>

	<Select
		id="appStateFilter"
		label="Filter by state"
		bind:value={appStateFilter}
		choices={appResp?.states?.map((state) => ({ id: state, value: state, label: title(state) }))}
	/>

	{#each appResp?.appResp?.apps as app, i}
		{#if (!appPositionFilter || appPositionFilter == app?.position) && (!appStateFilter || appStateFilter == app?.state)}
			<AppCard {app} index={i} />
			<div class="mb-3"></div>
		{/if}
	{/each}
{:catch err}
	<ErrorComponent msg={err?.toString() || 'Unknown error'} />
{/await}
