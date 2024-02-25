<script lang="ts">
	import type { PartialEntity } from '$lib/generated/arcadia/PartialEntity';
	import { panelState } from '$lib/panelState';
	import { title } from '$lib/strings';
	import Card from '../../../../components/Card.svelte';
	import CardLinkButton from '../../../../components/CardLinkButton.svelte';

	export let result: PartialEntity;

	const getType = (e: PartialEntity) => {
		if ('Bot' in e) {
			let bot = e.Bot;

			switch (bot?.type) {
				case 'claimed':
					return `Claimed by ${bot?.claimed_by}`;
				default:
					return title(bot?.type);
			}
		} else if ('Server' in e) {
			let server = e.Server;

			switch (server?.type) {
				case 'claimed':
					return `Claimed by ${server?.claimed_by}`; // TODO: add claimed_by to servers as well
				default:
					return title(server?.type);
			}
		} else {
			return 'Unknown';
		}
	};
</script>

{#if 'Bot' in result}
	<Card>
		<img slot="image" src={result?.Bot?.user?.avatar} alt="" />
		<svelte:fragment slot="display-name">{result?.Bot?.user?.username}</svelte:fragment>
		<svelte:fragment slot="short">{result?.Bot?.short}</svelte:fragment>
		<svelte:fragment slot="index">
			<slot name="index" />
		</svelte:fragment>
		<svelte:fragment slot="type">{getType(result)}</svelte:fragment>
		<svelte:fragment slot="actionA">
			<CardLinkButton
				target="_blank"
				link={`${$panelState?.core_constants?.frontend_url}/bots/${result?.Bot?.bot_id}`}
				showArrow={false}>View</CardLinkButton
			>
		</svelte:fragment>
		<svelte:fragment slot="actionB">
			<CardLinkButton
				target="_blank"
				link={`https://discord.com/api/v10/oauth2/authorize?client_id=${result?.Bot?.client_id}&permissions=0&scope=bot%20applications.commands&guild_id=${$panelState?.core_constants?.servers?.testing}`}
				showArrow={false}>Invite</CardLinkButton
			>
		</svelte:fragment>
		<svelte:fragment slot="extra">
			<slot name="extra" />
		</svelte:fragment>
	</Card>
{:else if 'Server' in result}
	<Card>
		<img slot="image" src={result?.Server?.avatar} alt="" />
		<svelte:fragment slot="display-name">{result?.Server?.name}</svelte:fragment>
		<svelte:fragment slot="short">{result?.Server?.short}</svelte:fragment>
		<svelte:fragment slot="index">
			<slot name="index" />
		</svelte:fragment>
		<svelte:fragment slot="type">{getType(result)}</svelte:fragment>
		<svelte:fragment slot="actionA">
			<CardLinkButton
				target="_blank"
				link={`${$panelState?.core_constants?.frontend_url}/servers/${result?.Server?.server_id}`}
				showArrow={false}>View</CardLinkButton
			>
		</svelte:fragment>
		<svelte:fragment slot="actionB">
			<CardLinkButton
				target="_blank"
				link={`${$panelState?.core_constants?.frontend_url}/servers/${result?.Server?.server_id}/invite`}
				showArrow={false}>Invite</CardLinkButton
			>
		</svelte:fragment>
		<svelte:fragment slot="extra">
			<slot name="extra" />
		</svelte:fragment>
	</Card>
{/if}
