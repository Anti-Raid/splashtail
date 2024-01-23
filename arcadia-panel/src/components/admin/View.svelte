<script lang="ts">
	import ErrorComponent from '../Error.svelte';
	import Loading from '../Loading.svelte';
	import type { Readable } from 'svelte/store';
	import { DataHandler, Datatable, Th, ThFilter } from '@vincjo/datatables';
	import ListItem from '../ListItem.svelte';
	import UnorderedList from '../UnorderedList.svelte';
	import type { Schema } from './types';
	import { castToArray, castToLink, fetchFields, validateDataWithFields } from './logic';
	import Manage from './Manage.svelte';
	import OrderedList from '../OrderedList.svelte';
	import Add from './Add.svelte';

	export let schema: Schema<any>;
	let rows: Readable<any[]>;
	let show = true;

	const fetchData = async () => {
		let viewData = await schema?.viewAll();

		if (!viewData) viewData = [];

		if (viewData.length > 0 && schema?.strictSchemaValidation) {
			validateDataWithFields(
				viewData[0],
				schema,
				await fetchFields('view', schema?.fields, 'strictSchemaValidation')
			);
		}

		let eRows = await schema?.viewToTable(viewData);

		const handler = new DataHandler(eRows.data, { rowsPerPage: 10 });
		rows = handler.getRows();

		let fields = await fetchFields('view', eRows.fields);
		let pkey = schema?.getPrimaryKey('view');

		return {
			handler,
			eRows,
			fields,
			pkey,
			viewData
		};
	};

	let currentlyOpenManageKey: string;
</script>

{#await fetchData()}
	<Loading msg="Fetching entries..." />
{:then data}
	{#if schema?.getCaps()?.includes('create')}
		<Add {schema} />
	{:else}
		<div class="mb-3"></div>
	{/if}

	<Datatable handler={data.handler} search={false}>
		<table>
			<thead>
				<tr>
					{#each data.fields as f}
						<Th handler={data.handler} orderBy={f.id}>{f.label}</Th>
					{/each}
					<Th handler={data.handler} orderBy={data.pkey}>Manage</Th>
				</tr>
				<tr>
					{#each data.fields as f}
						<ThFilter handler={data.handler} filterBy={f.id} />
					{/each}
					<ThFilter handler={data.handler} filterBy={data.pkey} />
				</tr>
			</thead>
			<tbody>
				{#each $rows as row}
					<tr>
						{#each data.fields as field}
							{#if field.renderMethod == 'text'}
								<td>
									{row[field.id]}
								</td>
							{:else if field.renderMethod == 'unordered-list'}
								<td>
									<UnorderedList>
										{#each castToArray(row[field.id]) as cols}
											<ListItem>
												{#if field.type == 'text[kv]'}
													{JSON.stringify(cols)}
												{:else if field.type == 'ibl:link'}
													<a
														class="text-blue-400 hover:text-blue-500"
														href={castToLink(cols)?.value}
														target="_blank">{castToLink(cols)?.name}</a
													>
												{:else}
													{cols}
												{/if}
											</ListItem>
										{/each}
									</UnorderedList>
								</td>
							{:else if field.renderMethod == 'ordered-list'}
								<td>
									<OrderedList>
										{#each castToArray(row[field.id]) as cols}
											<ListItem>
												{#if field.type == 'text[kv]'}
													{JSON.stringify(cols)}
												{:else if field.type == 'ibl:link'}
													<a
														class="text-blue-400 hover:text-blue-500"
														href={castToLink(cols)?.value}
														target="_blank">{castToLink(cols)?.name}</a
													>
												{:else}
													{cols}
												{/if}
											</ListItem>
										{/each}
									</OrderedList>
								</td>
							{:else if field.renderMethod == 'custom'}
								<td>
									{#if field?.customRenderer}
										{#await field?.customRenderer('view', row)}
											<p class="animate-pulse">Loading {field.id}</p>
										{:then data}
											{data}
										{/await}
									{:else}
										{row[field.id]}
									{/if}
								</td>
							{:else if field.renderMethod == 'custom[html]'}
								<td>
									{#if field?.customRenderer}
										{#await field?.customRenderer('view', row)}
											<p class="animate-pulse">Loading {field.id}</p>
										{:then data}
											{@html data}
										{/await}
									{:else}
										{@html row[field.id]}
									{/if}
								</td>
							{:else if field.renderMethod != 'none'}
								<p class="text-red-400">Unsupported render method: {field.renderMethod}</p>
							{/if}
						{/each}
						<td>
							<button
								class="inline-block py-4 px-3 text-xl text-themable-400 hover:bg-slate-700"
								on:click={() => {
									currentlyOpenManageKey = row[data.pkey];
									show = true;
								}}
							>
								Manage
							</button>
						</td>
					</tr>
				{/each}
				{#if currentlyOpenManageKey}
					<Manage
						bind:show
						data={{
							schema,
							manageData: data.viewData.find((v) => v?.[data.pkey] == currentlyOpenManageKey)
						}}
					/>
				{/if}
			</tbody>
		</table>
	</Datatable>
{:catch err}
	<ErrorComponent msg={`Failed to fetch schema view: ${err}`} />
{/await}

<style>
	table {
		color: white;
		width: 210%;
		margin: 0 !important;
	}
	tbody td {
		border: 1px solid #f5f5f5;
		padding: 4px 20px;
	}
	tbody tr {
		transition: all, 0.2s;
	}
	tbody tr:hover {
		background: #252323;
	}
</style>
