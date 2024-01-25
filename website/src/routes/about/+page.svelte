<script lang="ts">
	import Divider from '../../components/Divider.svelte';
	import BotFeatures from '../../components/common/BotFeatures.svelte';
	import ClusterHealth from '../../components/common/ClusterHealth.svelte';
	import support from "$lib/configs/data/support.json";

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
	<a href={support?.discord} class="text-indigo-600 xl:inline hover:text-red-600"
		>Discord Server</a
	> from threats, unsafe bots and spamming using our advanced technology!
</p>

<div class="p-4" />

<Divider text="Features" margin="6" />

<div class="p-2" />

<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
	<div class="lg:text-center" id="features">
		<h2 class="text-lg font-semibold text-indigo-600">Features</h2>
		<p class="max-w-2xl text-xl text-white lg:mx-auto">What features does AntiRaid offer?</p>
	</div>

	<div class="mt-10">
		<dl class="space-y-10 md:grid md:grid-cols-2 md:gap-x-8 md:gap-y-10 md:space-y-0">
			<BotFeatures />
		</dl>
	</div>
</div>

<div class="p-4" />

<Divider text="Meet the Team" margin="6" />

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
