<script>
	import Nightmare from '../components/Nightmare.svelte';
</script>

<Nightmare
	Title="Home"
	Description="This website is extremely experimental, and should not be used by the public at this time."
/>

<main class="mx-auto mt-10 max-w-7xl px-4 sm:mt-12 sm:px-6 md:mt-16 lg:mt-20 lg:px-8">
	<div class="text-center lg:text-left">
		<h1 class="text-4xl font-bold tracking-tight text-gray-900 sm:text-5xl md:text-6xl">
			<span class="block text-white xl:inline">Protect your</span>
			<a
				href="https://discord.gg/v9qRYthfHf"
				class="block text-indigo-600 xl:inline hover:text-red-600">Discord Server</a
			>
		</h1>

		<p
			class="mt-3 text-base text-white sm:mx-auto sm:mt-5 sm:max-w-xl sm:text-lg md:mt-5 md:text-xl lg:mx-0"
		>
			With our services, you can easily protect your <a
				href="https://discord.gg/v9qRYthfHf"
				class="font-bold tracking-tight text-indigo-600 hover:text-red-600">Discord Server</a
			> in a matter of seconds!
		</p>

		<div class="mt-5 sm:mt-8 flex justify-center items-center">
			<div class="rounded-md shadow">
				<a
					href="/invite"
					class="flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-8 py-3 text-base font-medium text-white hover:bg-indigo-700 md:py-4 md:px-10 md:text-lg"
					><i class="fa-solid fa-plus pr-2" /> Invite</a
				>
			</div>

			<div class="ml-2">
				<a
					href="/about"
					class="flex items-center justify-center rounded-md border border-transparent bg-indigo-100 px-8 py-3 text-base font-medium text-indigo-700 hover:bg-indigo-200 md:py-4 md:px-10 md:text-lg"
					>Learn More <i class="fa-solid fa-arrow-right pl-2" /></a
				>
			</div>
		</div>
	</div>
</main>

<div class="p-6" />

<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
	<div class="lg:text-center" id="features">
		<h2 class="text-lg font-semibold text-indigo-600">Features</h2>
		<p class="max-w-2xl text-xl text-white lg:mx-auto">What features does AntiRaid offer?</p>
	</div>

	<div class="mt-10">
		<dl class="space-y-10 md:grid md:grid-cols-2 md:gap-x-8 md:gap-y-10 md:space-y-0">
			<div class="relative">
				<dt>
					<div
						class="absolute flex h-12 w-12 items-center justify-center rounded-md bg-indigo-500 text-white"
					>
						<!-- Heroicon name: outline/globe-alt -->
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
								d="M12 21a9.004 9.004 0 008.716-6.747M12 21a9.004 9.004 0 01-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 017.843 4.582M12 3a8.997 8.997 0 00-7.843 4.582m15.686 0A11.953 11.953 0 0112 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0121 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0112 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 013 12c0-1.605.42-3.113 1.157-4.418"
							/>
						</svg>
					</div>
					<p class="ml-16 text-lg font-medium leading-6 text-white">Raid Prevention</p>
				</dt>
				<dd class="mt-2 ml-16 text-base text-white">
					This feature locks down your server, to help prevent members from joining when you are
					being raided.
				</dd>
			</div>

			<div class="relative">
				<dt>
					<div
						class="absolute flex h-12 w-12 items-center justify-center rounded-md bg-indigo-500 text-white"
					>
						<!-- Heroicon name: outline/scale -->
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
								d="M12 3v17.25m0 0c-1.472 0-2.882.265-4.185.75M12 20.25c1.472 0 2.882.265 4.185.75M18.75 4.97A48.416 48.416 0 0012 4.5c-2.291 0-4.545.16-6.75.47m13.5 0c1.01.143 2.01.317 3 .52m-3-.52l2.62 10.726c.122.499-.106 1.028-.589 1.202a5.988 5.988 0 01-2.031.352 5.988 5.988 0 01-2.031-.352c-.483-.174-.711-.703-.59-1.202L18.75 4.971zm-16.5.52c.99-.203 1.99-.377 3-.52m0 0l2.62 10.726c.122.499-.106 1.028-.589 1.202a5.989 5.989 0 01-2.031.352 5.989 5.989 0 01-2.031-.352c-.483-.174-.711-.703-.59-1.202L5.25 4.971z"
							/>
						</svg>
					</div>
					<p class="ml-16 text-lg font-medium leading-6 text-white">Automatic Roles</p>
				</dt>
				<dd class="mt-2 ml-16 text-base text-white">
					This feature automatically gives a role to new members of your server!
				</dd>
			</div>

			<div class="relative">
				<dt>
					<div
						class="absolute flex h-12 w-12 items-center justify-center rounded-md bg-indigo-500 text-white"
					>
						<!-- Heroicon name: outline/bolt -->
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
								d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z"
							/>
						</svg>
					</div>
					<p class="ml-16 text-lg font-medium leading-6 text-white">Advanced Infractions System</p>
				</dt>
				<dd class="mt-2 ml-16 text-base text-white">
					Our Advanced Infractions System allows you to easily make cases against a user for
					violating rules, while being able to make it contain information such as moderator name,
					allegation, reason, action and proof.
				</dd>
			</div>

			<div class="relative">
				<dt>
					<div
						class="absolute flex h-12 w-12 items-center justify-center rounded-md bg-indigo-500 text-white"
					>
						<!-- Heroicon name: outline/chat-bubble-bottom-center-text -->
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
								d="M7.5 8.25h9m-9 3H12m-9.75 1.51c0 1.6 1.123 2.994 2.707 3.227 1.129.166 2.27.293 3.423.379.35.026.67.21.865.501L12 21l2.755-4.133a1.14 1.14 0 01.865-.501 48.172 48.172 0 003.423-.379c1.584-.233 2.707-1.626 2.707-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0012 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018z"
							/>
						</svg>
					</div>
					<p class="ml-16 text-lg font-medium leading-6 text-white">Audit Logging</p>
				</dt>
				<dd class="mt-2 ml-16 text-base text-white">
					Everytime a action occurs on your server, we will send a message to a specific channel to
					help better protect your server against attackers.
				</dd>
			</div>
		</dl>
	</div>
</div>
