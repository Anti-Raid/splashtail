<script lang="ts">
	import { panelAuthState } from '$lib/panelAuthState';
	import Loading from '../../../../components/Loading.svelte';
	import ErrorComponent from '../../../../components/Error.svelte';
	import type { PartialEntity } from '$lib/generated/arcadia/PartialEntity';
	import Card from '../../../../components/Card.svelte';
	import { panelState } from '$lib/panelState';
	import CardLinkButton from '../../../../components/CardLinkButton.svelte';
	import Column from '../../../../components/Column.svelte';
	import { panelQuery } from '$lib/fetch';
	import type { RPCWebAction } from '$lib/generated/arcadia/RPCWebAction';
	import QueueAction from './QueueAction.svelte';

	const fetchQueueBots = async () => {
		let res = await panelQuery({
			BotQueue: {
				login_token: $panelAuthState?.loginToken || ''
			}
		});

		if (!res.ok) throw new Error('Failed to fetch bots in queue');

		let bots: PartialEntity[] = await res.json();

		let actionsRes = await panelQuery({
			GetRpcMethods: {
				login_token: $panelAuthState?.loginToken || '',
				filtered: true
			}
		});

		if (!actionsRes.ok) throw new Error('Failed to fetch actions');

		let actions: RPCWebAction[] = await actionsRes.json();

		let botsObj = [];

		for (let bot of bots) {
			if ('Bot' in bot) {
				botsObj.push(bot.Bot);
			}
		}

		return {
			bots: botsObj,
			actions
		};
	};
</script>

{#await fetchQueueBots()}
	<Loading msg={'Fetching bots in queue...'} />
{:then bots}
	<h2 class="text-3xl font-bold">Bot Queue</h2>

	<div class="p-3" />

	<Column>
		{#each bots.bots as bot, i}
			<Card>
				<img slot="image" src={bot?.user?.avatar} alt="" />
				<svelte:fragment slot="display-name">{bot?.user?.username}</svelte:fragment>
				<svelte:fragment slot="short">{bot?.short}</svelte:fragment>
				<svelte:fragment slot="index">#{i + 1}</svelte:fragment>
				<svelte:fragment slot="type">
					{bot?.claimed_by ? `Claimed by ${bot?.claimed_by}` : 'Pending Review'}
				</svelte:fragment>
				<svelte:fragment slot="actionA">
					<CardLinkButton
						target="_blank"
						link={`${$panelState?.core_constants?.frontend_url}/bots/${bot?.bot_id}`}
						showArrow={false}>View</CardLinkButton
					>
				</svelte:fragment>
				<svelte:fragment slot="actionB">
					<CardLinkButton
						target="_blank"
						link={`https://discord.com/api/v10/oauth2/authorize?client_id=${bot?.client_id}&permissions=0&scope=bot%20applications.commands&guild_id=${$panelState?.core_constants?.servers?.testing}`}
						showArrow={false}>Invite</CardLinkButton
					>
				</svelte:fragment>
				<svelte:fragment slot="extra">
					<QueueAction {bot} actions={bots.actions} />
				</svelte:fragment>
			</Card>
		{/each}
	</Column>
{:catch err}
	<ErrorComponent msg={`Failed to fetch bots in queue: ${err}`} />
{/await}
