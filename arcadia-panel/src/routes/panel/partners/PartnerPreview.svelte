<!--TODO, not yet done or enabled anywhere-->
<script lang="ts">
	import { panelQuery } from '$lib/fetch';
	import { panelAuthState } from '$lib/panelAuthState';
	import { panelState } from '$lib/panelState';
	import Card from '../../../components/Card.svelte';
	import CardLinkButton from '../../../components/CardLinkButton.svelte';
	import Column from '../../../components/Column.svelte';
	import ErrorComponent from '../../../components/Error.svelte';
	import Loading from '../../../components/Loading.svelte';
	import type { Partners } from '$lib/generated/arcadia/Partners';

	const fetchPartnerList = async () => {
		let res = await panelQuery({
			UpdatePartners: {
				login_token: $panelAuthState?.loginToken || '',
				action: 'List'
			}
		});

		if (!res.ok) throw new Error('Failed to fetch partner list');

		let partners: Partners = await res.json();

		let scopeRes = await panelQuery({
			GetMainCdnScope: {
				login_token: $panelAuthState?.loginToken || ''
			}
		});

		if (!scopeRes.ok) {
			let err = await scopeRes.text();
			throw new Error(`Failed to fetch main CDN scope: ${err}`);
		}

		let scope: string = await scopeRes.text();

		return {
			partners,
			scope
		};
	};
</script>

{#await fetchPartnerList()}
	<Loading msg="Fetching partner list..." />
{:then partners}
	<Column>
		{#each partners.partners.partners as partner, i}
			<Card>
				<img
					slot="image"
					src={`${$panelState?.core_constants?.cdn_url}/avatars/partners/${partner?.id}.webp`}
					alt=""
				/>
				<svelte:fragment slot="display-name">{partner?.name}</svelte:fragment>
				<svelte:fragment slot="short">{partner?.short}</svelte:fragment>
				<svelte:fragment slot="index">#{i + 1}</svelte:fragment>
				<svelte:fragment slot="type">{partner?.type}</svelte:fragment>
				<svelte:fragment slot="actionA">
					<CardLinkButton
						target="_blank"
						link={`${$panelState?.core_constants?.frontend_url}/about/partners`}
						showArrow={false}
						double={false}
					>
						View
					</CardLinkButton>
				</svelte:fragment>
			</Card>
		{/each}
	</Column>
{:catch err}
	<ErrorComponent msg={`Failed to fetch partner list: ${err}`} />
{/await}
