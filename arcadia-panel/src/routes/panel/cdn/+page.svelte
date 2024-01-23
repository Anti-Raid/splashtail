<script lang="ts">
	import { panelQuery } from '$lib/fetch';
	import { panelAuthState } from '$lib/panelAuthState';
	import Loading from '../../../components/Loading.svelte';
	import ErrorComponent from '../../../components/Error.svelte';
	import GreyText from '../../../components/GreyText.svelte';
	import type { CdnScopeData } from '$lib/generated/arcadia/CdnScopeData';

	const fetchCdnScopes = async () => {
		let res = await panelQuery({
			ListCdnScopes: {
				login_token: $panelAuthState?.loginToken || ''
			}
		});

		if (!res.ok) {
			let err = await res.text();
			throw new Error(`Failed to fetch CDN scopes: ${err}`);
		}

		let cdnScopes: { [key: string]: CdnScopeData } = await res.json();

		return cdnScopes;
	};
</script>

{#await fetchCdnScopes()}
	<Loading msg="Fetching CDN scopes..." />
{:then cdnScopes}
	<h1 class="text-3xl font-semibold">Choose a scope</h1>
	<GreyText
		>A scope is essentially a network share of a CDN on the server that is exposed to the panel!</GreyText
	>
	<div id="link-box" class="border rounded-md">
		{#each Object.entries(cdnScopes) as cdnScope, i}
			<a
				href={`/panel/cdn/${cdnScope[0]}`}
				class={`block rounded-t-md text-white hover:bg-slate-800 p-4 ${
					i < Object.entries(cdnScopes).length - 1 ? 'border-b' : 'rounded-md'
				}`}
			>
				{cdnScope[0]}
				<div class="mt-2 text-gray-400">
					<span class="font-semibold">Path: </span>{cdnScope[1].path}<br />
					<span class="font-semibold">Exposed URL: </span>{cdnScope[1].exposed_url}
				</div>
			</a>
		{/each}
	</div>
{:catch err}
	<ErrorComponent msg={`Failed to fetch CDN scopes: ${err?.toString()}`} />
{/await}
