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

	let navigation = [
		{ name: 'Home', href: '/', current: false },
		{ name: 'Invite', href: '/invite', current: false },
		{ name: 'About', href: '/about', current: false }
	];

	onMount(() => {
		navigation.map((p) => {
			if (p.href === $page.url.pathname) p.current = true;
		});
	})

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
</script>

<Update
	short="This site is experimental."
	long="This website is experimental, and may have issues."
/>

<nav>
	<div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
		<div class="relative flex h-16 items-center justify-between">
			<div class="inset-y-0 left-0 flex items-center">
				<button
					type="button"
					class="inline-flex items-center justify-center rounded-md p-2 text-white hover:bg-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
					on:click={() => mobileMenuOpen = !mobileMenuOpen}
					aria-controls="mobile-menu"
					aria-expanded="false"
				>
					<span class="sr-only">Open main menu</span>
					<svg
						class="block h-6 w-6"
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 24 24"
						stroke-width="1.5"
						stroke="currentColor"
						aria-hidden="true"
						id="menuIcon"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
						/>
					</svg>
					<svg
						class="hidden h-6 w-6"
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 24 24"
						stroke-width="1.5"
						stroke="currentColor"
						aria-hidden="true"
						id="closeIcon"
					>
						<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>
			<div class="flex flex-1 items-center justify-center sm:items-stretch sm:justify-start">
				<div class="flex flex-shrink-0 items-center">
					<img class="block h-8 w-auto lg:hidden" src="/logo.webp" alt="Antiraid" />

					<img class="hidden h-8 w-auto lg:block" src="/logo.webp" alt="Antiraid" />
				</div>
				<div class="hidden sm:ml-6 sm:block">
					<div class="flex space-x-4">
						{#each navigation as item}
							<a
								href={item.href}
								class={
									item.current ? 'bg-indigo-600 text-white' : 'text-white hover:bg-indigo-300 px-3 py-2 rounded-md text-sm font-medium'
								}
								aria-current={item.current ? 'page' : undefined}
							>
								{item.name}
							</a>
						{/each}
					</div>
				</div>
				<div
					class="absolute inset-y-0 right-0 flex items-center pr-2 sm:static sm:inset-auto sm:ml-6 sm:pr-0"
				>
					<div class="relative ml-3">
						{#await getLoginData()}
							<span class="w-auto flex items-center justify-center shadow-lg gap-x-2 shadow-themable-600/20 rounded-xl py-2.5 font-medium px-7 bg-gradient-to-tl from-themable-500 to-themable-700 text-white  hover:opacity-80 transition duration-200">
								<Icon icon="fa-solid:yin-yang" width="32px" class="animate-spin text-white" />
							</span>
						{:then data}
							{#if data && data?.user}
								<div>
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
									class="absolute right-0 z-10 mt-2 w-48 origin-top-right invisible"
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
									class="rounded-full p-1 text-gray-400 hover:text-white focus:outline-none"
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
			</div>
		</div>

		{#if mobileMenuOpen}
			<div id="mobile-menu">
				<div class="space-y-1 px-2 pt-2 pb-3">
					{#each navigation as item}
						<a
							href={item.href}
							class={item.current
								? 'bg-indigo-600 text-white'
								: 'text-gray-300 hover:bg-gray-700 hover:text-white block px-3 py-2 rounded-md text-base font-medium'
							}
							aria-current={item.current ? 'page' : undefined}
						>
							{item.name}
						</a>
					{/each}
				</div>
			</div>
		{/if}
	</div>
</nav>
