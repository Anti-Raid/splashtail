<script lang="ts">
	export let data: any;

	import Nightmare from '../../../components/Nightmare.svelte';
	import StepProgress from '../../../components/StepProgress.svelte';
	import Notice from '../../../components/Notice.svelte';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';

	const ImageLoadError = (a: any) => {
		a.target.src = '/logo.webp';
	};

	const CheckPerm = (permissions: any, owner: Boolean) => {
		if (permissions['Administrator'] || permissions['ManageGuild'] || owner === true) return true;
		else return false;
	};

	export let guild = data.user.guilds.find(
		(i: any) => i.id === data.slug && CheckPerm(i.permissions, i.owner)
	);

	let PageURL: String = '';
	onMount(() => (PageURL = `${window.location.protocol}//${window.location.host}`));

	let steps = [
		{
			name: 'Invite',
			current: true,
			completed: false,
			disableBack: true,
			onClick: () => {
				throw new Error(
					'You have not invited AntiRaid to your server. Please follow the instructions listed on the page!'
				);
			}
		},
		{
			name: 'Logging',
			current: false,
			completed: false,
			disableBack: true,
			onClick: () => {
				return true;
			}
		},
		{
			name: 'Advanced',
			current: false,
			completed: false,
			disableBack: false,
			onClick: () => {
				return true;
			}
		}
	];

	let currentStep: number = 0;
	if (
		$page.url.searchParams.get('complete_invite') &&
		$page.url.searchParams.get('complete_invite') === 'true'
	)
		currentStep = 1;

	const client_id = '858308969998974987';
	const state = JSON.stringify({
		session: crypto.randomUUID().replaceAll('-', ''),
		guild_id: guild.id,
		nextStep: 1
	});
</script>

{#if data.user}
	{#if guild}
		<Nightmare Title="Invite" Description="Invite AntiRaid into {guild.name}." />

		<div class="flex justify-center items-center mb-2">
			<img
				class="h-8 w-8 rounded-md"
				src="https://cdn.discordapp.com/icons/{guild.id}/{guild.icon}.png"
				height="32px"
				width="32px"
				alt="{guild.name}'s Server Image"
				on:error={ImageLoadError}
			/>

			<h3 class="ml-2 text-xl font-extrabold text-black dark:text-white truncate hover:underline">
				{guild.name}
			</h3>
		</div>

		<StepProgress bind:currentStep {steps}>
			{#if currentStep == 0}
				<h1 class="text-2xl text-white font-bold tracking-tight sm:text-5xl md:text-6xl">
					First, Let's invite <span class="text-red-600">AntiRaid</span>!
				</h1>
				<p class="block text-white font-semibold xl:inline">
					Once you invite the bot to your Discord Server, you will be redirected back here to
					continue setup!
				</p>

				<Notice
					Description="The <span class='text-gray-800 font-extrabold'>Administrator</span> permission is
                    required for AntiRaid to properly function."
				/>

				<div class="p-2" />

				<div class="flex justify-center items-center md:justify-normal md:items-start">
					<a
						class="mt-2 bg-indigo-600 px-3 py-2 text-white rounded-md text-base font-medium hover:cursor-pointer hover:bg-indigo-400"
						href="https://discord.com/api/oauth2/authorize?client_id={client_id}&response_type=code&permissions=8&scope=bot%20applications.commands&guild_id={guild.id}&disable_guild_select=true&state={state}&redirect_uri={PageURL}/invite/complete&prompt=consent"
						><i class="fa-solid fa-plus pr-2" /> Invite Now</a
					>
				</div>
			{/if}

			{#if currentStep == 1}
				<h1 class="text-2xl text-white font-bold tracking-tight sm:text-5xl md:text-6xl">
					Now, Let's configure <span class="text-red-600">Logging</span>!
				</h1>
				<p class="block text-white font-semibold xl:inline">
					Don't worry, this shouldn't be too hard and should only take a few moments!
				</p>

				<Notice
					Description="If you do not want Server Logging, or Audit Logs; you may simply skip this step!"
				/>

				<div class="p-2" />

				<fieldset class="border border-solid rounded-sm border-gray-300 p-3">
					<legend class="text-white font-bold tracking-tight">Channels</legend>
				</fieldset>
			{/if}

			{#if currentStep == 2}
				<h2 class="text-white font-black text-xl">Now, Let's get some advanced information!</h2>
			{/if}

			{#if currentStep == 3}
				<h2 class="text-white font-black text-base">Finished</h2>
			{/if}
		</StepProgress>
	{:else}
		<Nightmare Title="Invite" Description="Invite AntiRaid into your server." />
		<h1 class="text-white font-bold">You don't have permissions to invite us to this guild.</h1>
	{/if}
{:else}
	<Nightmare Title="Invite" Description="Invite AntiRaid into your server." />
	<h1 class="text-white font-bold">You are not logged in. Please login and reload this page.</h1>
{/if}
