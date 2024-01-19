<script lang="ts">
	import Divider from '../../components/Divider.svelte';

	interface TeamMember {
		Name: string;
		Role: string;
		Avatar: string;
	}

	const Members: TeamMember[] = [
		{
			Name: 'Ilieff',
			Role: 'Owner / CEO',
			Avatar:
				'https://cdn.discordapp.com/avatars/775855009421066262/7bea9898ea29e94bd23b7645b7c578c5.webp'
		},
		{
			Name: 'Select',
			Role: 'Co-Owner / Head Developer',
			Avatar:
				'https://cdn.discordapp.com/avatars/564164277251080208/b14fc83f7471961f1b415a4c22517e04'
		},
		{
			Name: 'Burger King',
			Role: 'Developer',
			Avatar:
				'https://cdn.discordapp.com/avatars/728871946456137770/490998f80f493f3861b5d4aa6f383b5c.webp'
		},
		{
			Name: 'Toxic Dev',
			Role: 'Popular Contributor',
			Avatar:
				'https://cdn.discordapp.com/avatars/510065483693817867/a_5b8b978d724408fa66bc880d1ecc8d17.gif'
		}
	];
</script>

<main class="mx-auto mt-10 max-w-7xl px-4 sm:mt-12 sm:px-6 md:mt-16 lg:mt-20 lg:px-8">
	<div class="text-center lg:text-left">
		<h1 class="text-4xl font-bold tracking-tight text-gray-900 sm:text-5xl md:text-6xl">
			<span class="block text-white xl:inline">&#128075; About Us</span>
		</h1>

		<p
			class="mt-3 text-base text-white sm:mx-auto sm:mt-5 sm:max-w-xl sm:text-lg md:mt-5 md:text-xl lg:mx-0"
		>
			Learn more about <span class="text-indigo-600 font-bold">AntiRaid</span> and our team!
		</p>
	</div>
</main>

<div class="p-4" />

<p class="text-xl text-white font-semibold text-center">
	<span class="text-indigo-600">AntiRaid</span> is a
	<span class="font-extrabold">Automatic Moderation Service</span>
	created to protect your
	<a href="https://discord.gg/v9qRYthfHf" class="text-indigo-600 xl:inline hover:text-red-600"
		>Discord Server</a
	> from threats, unsafe bots and spamming using our advanced technology!
</p>

<div class="p-4" />

<Divider Text="Features" Margin="6" />

<div class="p-2" />

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

<div class="p-4" />

<Divider Text="Meet the Team" Margin="6" />

<div class="py-10">
	<div class="mx-auto grid max-w-7xl gap-x-6 gap-y-10 px-6 lg:px-8 xl:grid-cols-3">
		<div class="max-w-2xl">
			<h2 class="text-3xl font-bold tracking-tight text-white sm:text-4xl">Meet the Team!</h2>
			<p class="mt-3 text-lg leading-8 text-white">
				Interested in joining our team? Join our <a
					href="https://discord.gg/v9qRYthfHf"
					class="block text-indigo-600 font-bold xl:inline hover:text-red-600">Discord Server</a
				>
			</p>
		</div>

		<ul class="grid gap-x-8 gap-y-12 sm:grid-cols-2 sm:gap-y-16 xl:col-span-2">
			{#each Members as Member}
				<li>
					<div class="flex items-center gap-x-6">
						<img class="h-16 w-16 rounded-full" src={Member.Avatar} alt={Member.Name} />
						<div>
							<h3 class="text-lg font-semibold leading-7 tracking-tight text-white">
								{Member.Name}
							</h3>
							<p class="text-md font-semibold leading-6 text-indigo-600">{Member.Role}</p>
						</div>
					</div>
				</li>
			{/each}
		</ul>
	</div>
</div>
