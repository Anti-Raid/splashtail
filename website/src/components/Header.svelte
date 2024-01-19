<script lang="ts">
	export let user: any;
	user = user;

	import Update from './Update.svelte';
	import Swal from 'sweetalert2';
	import { page } from '$app/stores';

	let navigation = [
		{ name: 'Home', href: '/', current: false },
		{ name: 'Invite', href: '/invite', current: false },
		{ name: 'About', href: '/about', current: false }
	];

	navigation.map((p) => {
		if (p.href === $page.url.pathname) p.current = true;
	});

	const profileNavigation = [
		{ name: 'Profile', href: '/profile' },
		{ name: 'Logout', href: '/auth/logout' }
	];

	const Alert = (title: string, description: string, time: number) => {
		Swal.fire({
			title: title,
			text: description,
			timer: time,
			timerProgressBar: true
		});
	};

	const classNames = (...classes: any) => {
		return classes.filter(Boolean).join(' ');
	};

	const loginDiscord = async () => {
		const data = await fetch('https://api.antiraid.xyz/auth/login').catch((error) => {
			Alert('Error:', error, 4000);
		});

		if (data.status === 200) {
			const json = await data.json();

			if (json.error) Alert('Error:', json.error, 4000);
			else window.location.href = json.url;
		} else Alert('Error:', `It seems that our servers is having issues at this time!`, 2000);
	};

	const openMobileMenu = () => {
		const menu = document.getElementById('mobile-menu') as HTMLDivElement;
		const menuIcon = document.getElementById('menuIcon') as HTMLElement;
		const closeIcon = document.getElementById('closeIcon') as HTMLElement;
		const currentClass = menu.className;

		if (currentClass === 'hidden') {
			menu.className = 'block';
			menuIcon.className.baseVal = 'hidden h-6 w-6';
			closeIcon.className.baseVal = 'block h-6 w-6';
		} else {
			menu.className = 'hidden';
			menuIcon.className.baseVal = 'block h-6 w-6';
			closeIcon.className.baseVal = 'hidden h-6 w-6';
		}
	};

	const openProfileMenu = () => {
		const profileMenu = document.getElementById('profile_menu') as HTMLDivElement;
		const className = profileMenu.className;

		// Open
		if (className === 'absolute right-0 z-10 mt-2 w-48 origin-top-right invisible')
			profileMenu.className =
				'absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none';
		// Close
		else profileMenu.className = 'absolute right-0 z-10 mt-2 w-48 origin-top-right invisible';
	};

	const openNotificationPanel = () => {
		const notificationPanel = document.getElementById('open-notifications') as HTMLDivElement;
		const className = notificationPanel.className;

		// Open
		if (className === 'absolute right-0 z-10 mt-2 w-48 origin-top-right invisible')
			notificationPanel.className =
				'absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none';
		// Close
		else notificationPanel.className = 'absolute right-0 z-10 mt-2 w-48 origin-top-right invisible';
	};

	let notificationData = null;
	if (user) notificationData = user.notifications.filter((i: any) => i.read === false);
	export let notifications = notificationData;
</script>

<Update
	short="This site is experimental."
	long="This website is experimental, and may have issues."
/>

<nav>
	<div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
		<div class="relative flex h-16 items-center justify-between">
			<div class="absolute inset-y-0 left-0 flex items-center sm:hidden">
				<button
					type="button"
					class="inline-flex items-center justify-center rounded-md p-2 text-white hover:bg-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
					on:click={openMobileMenu}
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
								class={classNames(
									item.current ? 'bg-indigo-600 text-white' : 'text-white hover:bg-indigo-300',
									'px-3 py-2 rounded-md text-sm font-medium'
								)}
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
					{#if user}
						<button
							type="button"
							class="rounded-full p-1 text-white hover:text-gray-300 focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800"
							id="notifications-button"
							aria-expanded="false"
							aria-haspopup="true"
							on:click={openNotificationPanel}
						>
							<span class="sr-only">View notifications</span>
							<svg
								class="h-6 w-6"
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								aria-hidden="true"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M14.857 17.082a23.848 23.848 0 005.454-1.31A8.967 8.967 0 0118 9.75v-.7V9A6 6 0 006 9v.75a8.967 8.967 0 01-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 01-5.714 0m5.714 0a3 3 0 11-5.714 0"
								/>
							</svg>
						</button>
					{/if}

					<div class="relative ml-3">
						{#if user}
							<div>
								<button
									type="button"
									class="flex rounded-full hover:bg-gray-200 text-white hover:text-gray-300 text-sm focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800"
									id="user-menu-button"
									aria-expanded="false"
									aria-haspopup="true"
									on:click={openProfileMenu}
								>
									<span class="sr-only">Open user menu</span>
									<img
										class="h-8 w-8 rounded-full"
										src="https://cdn.discordapp.com/avatars/{user.id}/{user.discordUser.avatar}"
										alt=""
									/>
								</button>
							</div>

							<div
								class="absolute right-0 z-10 mt-2 w-48 origin-top-right invisible"
								role="menu"
								aria-orientation="vertical"
								aria-labelledby="notifications-button"
								tabindex="-1"
								id="open-notifications"
							>
								<h2>Notifications</h2>
								<button>Mark all as Read!</button>

								{#if notifications.length === 0}
									<h2>There are no notifications to show!</h2>
								{:else}
									<h2>There are some notifications to show!</h2>
								{/if}
							</div>

							<div
								class="absolute right-0 z-10 mt-2 w-48 origin-top-right invisible"
								role="menu"
								aria-orientation="vertical"
								aria-labelledby="user-menu-button"
								tabindex="-1"
								id="profile_menu"
							>
								{#each profileNavigation as item}
									<a href={item.href} class="block px-4 py-2 text-sm text-gray-700">
										{item.name}
									</a>
								{/each}
							</div>
						{:else}
							<button
								type="button"
								on:click={loginDiscord}
								class="rounded-ful p-1 text-gray-400 hover:text-white focus:outline-none"
							>
								Login
							</button>
						{/if}
					</div>
				</div>
			</div>
		</div>

		<div class="hidden" id="mobile-menu">
			<div class="space-y-1 px-2 pt-2 pb-3">
				{#each navigation as item}
					<a
						href={item.href}
						class={classNames(
							item.current
								? 'bg-indigo-600 text-white'
								: 'text-gray-300 hover:bg-gray-700 hover:text-white',
							'block px-3 py-2 rounded-md text-base font-medium'
						)}
						aria-current={item.current ? 'page' : undefined}
					>
						{item.name}
					</a>
				{/each}
			</div>
		</div>
	</div>
</nav>
