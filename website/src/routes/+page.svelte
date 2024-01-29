<script>
	import Meta from '../components/Meta.svelte';
	import Icon from '@iconify/svelte';
	import support from '../lib/configs/data/support.json';
	import BotFeatures from '../components/common/BotFeatures.svelte';
	import ClusterHealth from '../components/common/ClusterHealth.svelte';
	import Message from "../components/Message.svelte";
	import { makeSharedRequest, opGetClusterHealth } from '$lib/fetch/ext';
	import CommandList from '../components/common/CommandList.svelte';
</script>

<Meta
	title="Home"
	description="This website is extremely experimental, and should not be used by the public at this time."
/>

<div class="text-center lg:text-left">
	<h1 class="text-4xl font-bold tracking-tight text-gray-900 sm:text-5xl md:text-6xl">
		<span class="block text-white xl:inline">Protect your</span>
		<a
			href={support?.discord}
			class="block text-indigo-600 xl:inline hover:text-red-600">Discord Server</a
		>
	</h1>

	<p
		class="mt-3 text-base text-white sm:mx-auto sm:mt-5 sm:max-w-xl sm:text-lg md:mt-5 md:text-xl lg:mx-0"
	>
		With our services, you can easily protect your <a
			href={support?.discord}
			class="font-bold tracking-tight text-indigo-600 hover:text-red-600">Discord Server</a
		> in a matter of seconds!
	</p>

	<div class="mt-5 sm:mt-8 flex justify-center items-center">
		<div class="rounded-md shadow">
			<a
				href="/invite"
				class="flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-8 py-3 text-base font-medium text-white hover:bg-indigo-700 md:py-4 md:px-10 md:text-lg"
			>
				<Icon icon="mdi:plus" /> Invite
			</a>
		</div>

		<div class="ml-2">
			<a
				href="/about"
				class="flex items-center justify-center rounded-md border border-transparent bg-indigo-100 px-8 py-3 text-base font-medium text-indigo-700 hover:bg-indigo-200 md:py-4 md:px-10 md:text-lg"
				>Learn More <Icon icon="fa-solid:arrow-right" class="pl-1 inline-block w-5" />
		</div>
	</div>
</div>

<div class="m-6" />

<div class="lg:text-center" id="features">
	<h2 class="text-lg font-semibold text-indigo-600">Features</h2>
	<p class="max-w-2xl text-xl text-white lg:mx-auto">What features does AntiRaid offer?</p>
</div>

<div class="mt-10">
	<dl class="space-y-10 md:grid md:grid-cols-2 md:gap-x-8 md:gap-y-10 md:space-y-0">
		<BotFeatures />
	</dl>
</div>

<hr class="my-10" />

<h2 class="text-4xl font-bold tracking-tight text-gray-900">
	<span class="block text-white xl:inline">Cluster Health</span>
</h2>	

{#await makeSharedRequest(opGetClusterHealth)}
	<Message type="loading">Fetching cluster data...</Message>
{:then data}
	<ClusterHealth instanceList={data} />
{:catch err}
	<Message type="error">Error loading cluster data: {err}</Message>
{/await}

<div class="mb-6" /> <!--TODO: Experiment with this a bit more-->

<h2 class="text-4xl font-bold tracking-tight text-gray-900">
	<span class="block text-white xl:inline">Command List</span>
</h2>	

{#await makeSharedRequest(opGetClusterHealth)}
	<Message type="loading">Fetching command list</Message>
{:then data}
	<CommandList instanceList={data} />
{:catch err}
	<Message type="error">Error loading cluster data: {err}</Message>
{/await}
