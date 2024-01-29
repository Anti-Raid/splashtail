<script lang="ts">
	import Update from './Update.svelte';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { getAuthCreds } from '../lib/auth/getAuthCreds';
	import { checkAuthCreds } from '../lib/auth/checkAuthCreds';
	import { logoutUser } from '../lib/auth/logoutUser';
	import { getUser } from '../lib/auth/getUser';
	import { User } from '../lib/generated/types';
	import logger from '../lib/ui/logger';
	import Icon from '@iconify/svelte';
	import NavButton from './inputs/button/NavButton.svelte';

	let navigation = [
		{ name: 'Home', href: '/' },
		{ name: 'Invite', href: '/invite' },
		{ name: 'About', href: '/about' }
	];

	let open = "";

	let mobileMenuOpen: boolean = false
	let profileMenuOpen: boolean = false

	type LoginData = null | {
		profileNavigation: {
			name: string
			href: string
		}[]
		user: User
	}

	let cachedLoginData: LoginData = null
	const getLoginData = async () => {
		if(cachedLoginData) {
			return cachedLoginData
		}

		let authCreds = getAuthCreds();

		if(!authCreds) return;

		let authCheck = false;
		
		try {
			authCheck = await checkAuthCreds(authCreds);
		} catch {}

		if(!authCheck) {
			logoutUser()
			return
		}

		let user = await getUser(authCreds);

		if(!user) {
			logger.error("Auth", "Failed to get user data")
			return
		}

		let data = {
			profileNavigation: [],
			user
		}

		cachedLoginData = data

		return data
	}

	const loginDiscord = async () => {
		// ...
	}

	$: {
		navigation.map((p) => {
			if (p.href === $page.url.pathname) open = p.name;
		});
	}
</script>

<Update
	short="This site is experimental."
	long="This website is experimental, and may have issues."
/>

<header class="top-0 w-full">
	<div class="max-w-7xl px-3 mx-auto py-3 flex items-center justify-between">
		<a href="/">
			<div class="flex items-center space-x-1">
				<img class="h-8 w-auto" src="/logo.webp" alt="Antiraid" />
				<p class="invisible md:visible text-xl text-white font-semibold">
					<span class="text-xl font-bold text-white">AntiRaid</span>
				</p>
			</div>
		</a>
		<div class="flex items-center space-x-2 relative">
			<div class="flex space-x-4">
				{#each navigation as item}
					<NavButton
						title={item.name}
						href={item.href}
						current={item.name === open}
						onClick={() => {
							mobileMenuOpen = false
						}}
						extClass="hidden md:block"
					/>
				{/each}
			</div>
		</div>
		<div class="flex items-center space-x-4">
			<button
				type="button"
				class="block md:hidden rounded-md p-2 font-medium text-left text-gray-300 hover:bg-slate-800 hover:text-white focus:outline-none focus:ring-1 focus:ring-inset focus:ring-white"
				on:click={() => mobileMenuOpen = !mobileMenuOpen}
				aria-controls="mobile-menu"
				aria-expanded="false"
			>
				<span class="sr-only">Open main menu</span>
				{#if mobileMenuOpen}
					<Icon icon="fa-solid:times" width="12px" />
				{:else}
					<Icon icon="fa-solid:bars" width="16px" />
				{/if}
			</button>
			{#await getLoginData()}
				<span class="w-auto flex items-center justify-center shadow-lg gap-x-2 shadow-themable-600/20 rounded-xl py-2.5 font-medium px-7 bg-gradient-to-tl from-themable-500 to-themable-700 text-white  hover:opacity-80 transition duration-200">
					<Icon icon="fa-solid:yin-yang" width="32px" class="animate-spin text-white" />
				</span>
			{:then data}
				{#if data && data?.user}
					<div class="w-full">
						<button
							type="button"
							class="flex rounded-full hover:bg-gray-200 text-white hover:text-gray-300 text-sm focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800"
							id="user-menu-button"
							aria-expanded="false"
							aria-haspopup="true"
							on:click={() => profileMenuOpen = !profileMenuOpen}
						>
							<span class="sr-only">Open user menu</span>
							<img
								class="h-8 w-8 rounded-full"
								src={data?.user?.user?.avatar}
								alt=""
							/>
						</button>
					</div>

					<div
						role="menu"
						aria-orientation="vertical"
						aria-labelledby="user-menu-button"
						tabindex="-1"
						id="profile_menu"
					>
						{#each (data?.profileNavigation || []) as item}
							<a href={item.href} class="block px-4 py-2 text-sm text-gray-700">
								{item.name}
							</a>
						{/each}
					</div>
				{:else}
					<button
						type="button"
						on:click={loginDiscord}
						class="px-4 py-2 text-sm font-medium text-left text-gray-50 rounded-lg cursor-pointer bg-indigo-600 hover:bg-indigo-800 focus:outline-none focus:ring-1 focus:ring-inset focus:ring-white"
					>
						Login
					</button>
				{/if}
			{:catch}
				<button
					type="button"
					on:click={() => {
						window.location.reload()
					}}
					class="text-red-500"
				>
					Reload?
				</button>
			{/await}	
		</div>
	</div>

	{#if mobileMenuOpen}
		<div id="mobile-menu" class="md:hidden">
			<div class="space-y-1 px-2 pt-2 pb-3">
				{#each navigation as item}
					<NavButton
						title={item.name}
						href={item.href}
						current={item.name === open}
						onClick={() => {
							mobileMenuOpen = false
						}}
						extClass="block"
					/>
				{/each}
			</div>
		</div>
	{/if}	
</header>
