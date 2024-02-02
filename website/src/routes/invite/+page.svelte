<script lang="ts">
	import ServerCard from '../../components/dashboard/ServerCard.svelte';
	import Meta from '../../components/Meta.svelte';

	export let data: any;

	const Invite = (id: string) => {
		return {
			name: 'Invite',
			click: () => {
				window.location.href = `/invite/${id}`;
			},
			icon: "mdi:discord"
		};
	};
</script>

<Meta title="Invite" description="Invite our bot into your server." />

{#if data.user}
	<div class="grid gap-3 md:gap-4 md:grid-cols-3 md:grid-rows-3">
		{#each data.user.guilds as guild}
			{#if guild.owner === true}
				<ServerCard
					id={guild.id}
					name={guild.name}
					image="https://cdn.discordapp.com/icons/{guild.id}/{guild.icon}.png"
					mainAction={Invite(guild.id)}
				/>
			{:else if guild.permissions['Administrator'] === true}
				<ServerCard
					id={guild.id}
					name={guild.name}
					image="https://cdn.discordapp.com/icons/{guild.id}/{guild.icon}.png"
					mainAction={Invite(guild.id)}
				/>
			{:else if guild.permissions['ManageGuild'] === true}
				<ServerCard
					id={guild.id}
					name={guild.name}
					image="https://cdn.discordapp.com/icons/{guild.id}/{guild.icon}.png"
					mainAction={Invite(guild.id)}
				/>
			{/if}
		{/each}
	</div>
{:else}
	<h2 class="text-white">You are not logged in. Please login to view this page.</h2>
{/if}
