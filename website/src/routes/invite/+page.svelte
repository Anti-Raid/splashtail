<script lang="ts">
	import ServerCard from '../../components/ServerCard.svelte';
	import Nightmare from '../../components/Nightmare.svelte';

	export let data: any;

	const Invite = (id: string) => {
		return {
			name: 'Invite',
			click: () => {
				window.location.href = `/invite/${id}`;
			}
		};
	};
</script>

<Nightmare Title="Invite" Description="Invite our bot into your server." />

{#if data.user}
	<div class="grid gap-3 md:gap-4 md:grid-cols-3 md:grid-rows-3">
		{#each data.user.guilds as guild}
			{#if guild.owner === true}
				<ServerCard
					name="guild"
					title={guild.name}
					image="https://cdn.discordapp.com/icons/{guild.id}/{guild.icon}.png"
					button={Invite(guild.id)}
				/>
			{:else if guild.permissions['Administrator'] === true}
				<ServerCard
					name="guild"
					title={guild.name}
					image="https://cdn.discordapp.com/icons/{guild.id}/{guild.icon}.png"
					button={Invite(guild.id)}
				/>
			{:else if guild.permissions['ManageGuild'] === true}
				<ServerCard
					name="guild"
					title={guild.name}
					image="https://cdn.discordapp.com/icons/{guild.id}/{guild.icon}.png"
					button={Invite(guild.id)}
				/>
			{/if}
		{/each}
	</div>
{:else}
	<h2 class="text-white">You are not logged in. Please login to view this page.</h2>
{/if}
