<script lang="ts">
	import StepProgress from '../../../../components/StepProgress.svelte';
	import { panelAuthState } from '$lib/panelAuthState';
	import Loading from '../../../../components/Loading.svelte';
	import ErrorComponent from '../../../../components/Error.svelte';
	import { error } from '$lib/toast';
	import { panelQuery } from '$lib/fetch';
	import type { RPCWebAction } from '$lib/generated/arcadia/RPCWebAction';
	import Column from '../../../../components/Column.svelte';
	import { panelState } from '$lib/panelState';
	import Select from './Select.svelte';
	import Modal from '../../../../components/Modal.svelte';
	import RPC from '../../../../components/rpc/RPC.svelte';
	import { page } from '$app/stores';
	import type { TargetType } from '$lib/generated/arcadia/TargetType';
	import { afterNavigate } from '$app/navigation';
	import type { PartialEntity } from '$lib/generated/arcadia/PartialEntity';
	import PartialEntityCard from './PartialEntityCard.svelte';

	let query: string;
	let selectedEntity: PartialEntity;
	let selectedId: number | null = null;
	let results: PartialEntity[] = [];

	let steps = [
		{
			name: 'Find',
			current: true,
			onClick: () => {
				if (!selectedId && selectedId !== 0) {
					throw new Error('No entity selected');
				}

				selectedEntity = results[selectedId];
			}
		},
		{
			name: 'Confirm',
			onClick: () => {
				if (!selectedEntity) {
					throw new Error('No entity selected');
				}

				modalVisible = true;
			}
		},
		{
			name: 'Action'
		}
	];

	let currentStep: number = 0;
	let modalVisible: boolean = false;

	const fetchRpcMethods = async () => {
		let actionsRes = await panelQuery({
			GetRpcMethods: {
				login_token: $panelAuthState?.loginToken || '',
				filtered: true
			}
		});

		if (!actionsRes.ok) throw new Error('Failed to fetch actions');

		let actions: RPCWebAction[] = await actionsRes.json();

		return {
			actions
		};
	};

	const searchEntity = async () => {
		let targetType = $page.params.targetType?.toString();

		if (!$panelState?.target_types?.includes(targetType as TargetType)) {
			error('This target type is not supported!');
			return false;
		}

		if (!query) {
			error('Please enter a search query');
			return false;
		}

		let res = await panelQuery({
			SearchEntitys: {
				login_token: $panelAuthState?.loginToken || '',
				query: query,
				target_type: targetType as TargetType
			}
		});

		if (!res.ok) {
			let err = await res.text();

			error(err || 'Unknown error while fetching');
			return false;
		}

		let resultsJson: PartialEntity[] = await res.json();

		if (resultsJson?.length == 0) {
			error('Could not find bots matching this query');
			results = [];
			return false;
		}

		results = resultsJson;

		return true;
	};

	const getRpcData = () => {
		let initialData: { [key: string]: any } = {};

		if ('Bot' in selectedEntity) {
			initialData = {
				target_id: selectedEntity?.Bot?.bot_id
			};
		} else if ('Server' in selectedEntity) {
			initialData = {
				target_id: selectedEntity?.Server?.server_id
			};
		} else {
			throw new Error('Unknown entity type');
		}

		return {
			targetType: $page.params.targetType as TargetType,
			initialData
		};
	};

	afterNavigate(() => {
		query = '';
		results = [];
		selectedId = null;
	});
</script>

{#await fetchRpcMethods()}
	<Loading msg={'Fetching available actions...'} />
{:then meta}
	{#key currentStep}
		<StepProgress {steps} bind:currentStep>
			{#if currentStep == 0 || (!selectedId && selectedId != 0) || !selectedEntity}
				<h2 class="text-white dark:text-gray-400 font-black text-xl">Let's get started!</h2>
				<p class="text-base text-white dark:text-gray-400 font-bold">
					Let's find what {$page?.params?.targetType?.toLowerCase()} you are taking action on!
				</p>

				<div class="p-2" />

				<div id="findEntity">
					<label for="searchBar" class="mb-2 text-sm font-medium text-white sr-only"
						>Let's find what {$page?.params?.targetType?.toLowerCase()} you are taking action on!</label
					>

					<div class="relative">
						<div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
							<svg
								class="w-4 h-4 text-gray-500 dark:text-gray-400"
								aria-hidden="true"
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 20 20"
							>
								<path
									stroke="currentColor"
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z"
								/>
							</svg>
						</div>

						<input
							type="search"
							bind:value={query}
							on:keydown={(e) => {
								if (e.key == 'Enter') {
									try {
										searchEntity();
									} catch (err) {
										error(err?.toString() || `Unknown error: ${e}`);
									}
								}
							}}
							id="searchBar"
							name="searchBar"
							class="block w-full p-4 pl-10 text-sm text-gray-900 border border-gray-300 rounded-lg bg-gray-50 focus:ring-slbg focus:border-slbg dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-slbg dark:focus:border-slbg"
							placeholder="What are you searching for?"
						/>

						<button
							type="submit"
							class="absolute right-2.5 top-2 bottom-2.5 bg-themable-500 text-themable-100 focus:ring-4 focus:outline-none focus:ring-themable-400 rounded-lg px-4"
							on:click={searchEntity}>Search</button
						>
					</div>

					{#if results}
						<div class="p-3" />

						<Column>
							{#each results as result, i}
								<PartialEntityCard {result}>
									<svelte:fragment slot="index">#{i + 1}</svelte:fragment>
									<svelte:fragment slot="extra">
										<Select index={i} bind:selected={selectedId} />
									</svelte:fragment>
								</PartialEntityCard>
							{/each}
						</Column>
					{:else}
						<p class="font-semibold text-xl text-red-500">
							There are no {$page.params.targetType} matching your query! Try making another search?
						</p>
					{/if}
				</div>
			{:else if currentStep == 1 && selectedEntity}
				<h2 class="text-white font-black text-xl">
					Alright! Let's make sure we have the right {$page.params.targetType} in mind!
				</h2>

				<div class="p-3" />

				<PartialEntityCard result={selectedEntity}>
					<svelte:fragment slot="index">#1</svelte:fragment>
					<svelte:fragment slot="extra">Selected</svelte:fragment>
				</PartialEntityCard>
			{:else if currentStep == 2}
				<h2 class="text-white font-black text-xl">Ready, set, action!</h2>

				{#if modalVisible}
					<Modal bind:showModal={modalVisible}>
						<h1 slot="header" class="font-semibold text-2xl">Perform RPC Action</h1>

						<RPC
							actions={meta?.actions}
							targetType={getRpcData().targetType}
							initialData={getRpcData().initialData}
						/>
					</Modal>
				{:else}
					<h2 class="text-white font-black text-base">
						You have completed this RPC Action! If you would like to perform another one, click the
						button down below.
					</h2>

					<button
						class="ml-2 bg-themable-600 text-lg px-4 text-themable-100 p-2 border-none rounded-md focus:ring-4 focus:outline-none focus:ring-themable-400"
						on:click={() => {
							currentStep = 0;
							modalVisible = true;
						}}>Perform Another!</button
					>
				{/if}
			{/if}
		</StepProgress>
	{/key}
{:catch err}
	<ErrorComponent msg={`Failed to fetch bots: ${err}`} />
{/await}
