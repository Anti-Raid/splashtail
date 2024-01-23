<script lang="ts">
	import { logoutUser } from '$lib/logout';
	import { panelAuthState } from '$lib/panelAuthState';

	interface MenuItem {
		Name: String;
		Href: string | (() => boolean);
	}

	let Navigation: MenuItem[] = [
		{
			Name: 'Home',
			Href: '/'
		}
	];

	$: {
		Navigation = [
			{
				Name: 'Home',
				Href: '/'
			}
		];

		if ($panelAuthState?.loginToken)
			Navigation = [
				...Navigation,
				{
					Name: 'Logout',
					Href: () => {
						logoutUser(true);
						return true;
					}
				}
			];
	}

	const onClickMenu = () => {
		const menu: HTMLElement | null = document.getElementById('menu');
		const menuIcon: any = document.getElementById('menuIcon') as HTMLElement;
		const closeIcon: any = document.getElementById('closeIcon') as HTMLElement;

		if (menu?.classList.contains('hidden')) {
			menu.classList.replace('hidden', 'block');
			menuIcon.className.baseVal = 'hidden h-6 w-6';
			closeIcon.className.baseVal = 'block h-6 w-6';
		} else {
			menu?.classList.replace('block', 'hidden');
			menuIcon.className.baseVal = 'block h-6 w-6';
			closeIcon.className.baseVal = 'hidden h-6 w-6';
		}
	};
</script>

<nav class="border-gray-200 px-2 sm:px-4 rounded">
	<div class="flex flex-wrap justify-between items-center mx-auto">
		<a href="/" class="flex items-center">
			<img
				src="https://cdn.infinitybots.gg/core/full_logo.webp"
				class="mr-3 h-6 sm:h-9"
				alt="IBL Logo"
			/>
			<span class="self-center text-xl font-semibold whitespace-nowrap text-white"
				>Infinity Panel</span
			>
		</a>

		<button
			on:click={onClickMenu}
			type="button"
			class="inline-flex items-center p-2 ml-3 text-sm text-gray-200 hover:text-gray-400 rounded-lg focus:outline-none"
			aria-controls="navbar-default"
			aria-expanded="false"
		>
			<span class="sr-only">Open main menu</span>
			<svg
				class="w-6 h-6"
				aria-hidden="true"
				fill="currentColor"
				viewBox="0 0 20 20"
				xmlns="http://www.w3.org/2000/svg"
				id="menuIcon"
				><path
					fill-rule="evenodd"
					d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z"
					clip-rule="evenodd"
				/></svg
			>

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
</nav>

<div id="menu" class="hidden bg-gray-700 rounded-b-md">
	<div class="px-2 pt-2 pb-3 space-y-1 sm:px-3">
		{#each Navigation as item}
			{#if typeof item.Href === 'string'}
				<a
					href={item.Href}
					class="block px-3 py-2 text-base font-medium bg-gray-400 text-black rounded-md hover:bg-gray-300"
				>
					{item.Name}
				</a>
			{:else}
				<button
					on:click={item.Href}
					class="text-left w-full block px-3 py-2 text-base font-medium bg-gray-400 text-black rounded-md hover:bg-gray-300"
				>
					{item.Name}
				</button>
			{/if}
		{/each}
	</div>
</div>
