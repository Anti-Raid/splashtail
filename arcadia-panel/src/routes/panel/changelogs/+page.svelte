<script lang="ts">
	import { panelQuery } from '$lib/fetch';
	import { panelAuthState } from '$lib/panelAuthState';
	import { panelState } from '$lib/panelState';
	import Loading from '../../../components/Loading.svelte';
	import type { ChangelogEntry } from '$lib/generated/arcadia/ChangelogEntry';
	import type {
		BaseSchema,
		Capability,
		CustomAction,
		Entry,
		FieldFetch,
		Schema
	} from '../../../components/admin/types';
	import logger from '$lib/logger';
	import View from '../../../components/admin/View.svelte';
	import { newField } from '../../../components/admin/helpers';
	import { build, hasPerm } from '$lib/perms';

	/* 
export interface ChangelogEntry { 
	version: string, 
	added: Array<string>, 
	updated: Array<string>, 
	removed: Array<string>, 
	github_html: string | null, 
	created_at: string, 
	extra_description: string, 
	prerelease: boolean, 
	published: boolean, 
}
	*/

	class ChangelogSchema implements BaseSchema<ChangelogEntry>, Schema<ChangelogEntry> {
		name: string = 'changelog';
		fields: FieldFetch<ChangelogEntry> = [
			{
				id: 'version',
				label: 'Version',
				type: 'text',
				helpText: 'The version of the changelog entry',
				required: true,
				disabled: false,
				renderMethod: 'text'
			},
			async (cap) => {
				return {
					id: 'added',
					label: 'Added',
					arrayLabel: 'Added Features',
					type: 'text[]',
					helpText: 'ABC was added...',
					required: true,
					disabled: false,
					renderMethod: cap == 'view' ? 'unordered-list' : 'text'
				};
			},
			async (cap) => {
				return {
					id: 'updated',
					label: 'Updated',
					arrayLabel: 'Updated Features',
					type: 'text[]',
					helpText: 'ABC was updated...',
					required: true,
					disabled: false,
					renderMethod: cap == 'view' ? 'unordered-list' : 'text'
				};
			},
			async (cap) => {
				return {
					id: 'removed',
					label: 'Removed',
					arrayLabel: 'Removed Features',
					type: 'text[]',
					helpText: 'ABC was removed...',
					required: true,
					disabled: false,
					renderMethod: cap == 'view' ? 'unordered-list' : 'text'
				};
			},
			async (cap) => {
				if (cap != 'create') {
					return {
						id: 'github_html',
						label: 'Github HTML',
						type: 'textarea',
						helpText: 'Github HTML for the changelog entry',
						required: false,
						disabled: false,
						renderMethod: 'text'
					};
				}
				return null;
			},
			{
				id: 'extra_description',
				label: 'Extra Description',
				type: 'textarea',
				helpText: 'Extra description for the changelog entry',
				required: false,
				disabled: false,
				renderMethod: 'text'
			},
			{
				id: 'prerelease',
				label: 'Prerelease',
				type: 'boolean',
				helpText: 'Is this a prerelease?',
				required: false,
				disabled: false,
				renderMethod: 'text'
			},
			async (cap: Capability) => {
				if (cap == 'create') return null;
				return newField('published', 'Published', 'Is this published?', false, false);
			},
			async (cap: Capability) => {
				if (cap == 'create') return null;
				return newField(
					'created_at',
					'Created At',
					'The date the changelog entry was created',
					false,
					true
				);
			}
		];

		strictSchemaValidation: boolean = true;
		strictSchemaValidationIgnore: string[] = [];

		getCaps(): Capability[] {
			let perms: Capability[] = ['view']; // All staff can view changelog entries
			if (hasPerm($panelState?.staff_member?.resolved_perms || [], build('changelogs', 'create'))) {
				perms.push('create');
			}
			if (hasPerm($panelState?.staff_member?.resolved_perms || [], build('changelogs', 'update'))) {
				perms.push('update');
			}
			if (hasPerm($panelState?.staff_member?.resolved_perms || [], build('changelogs', 'delete'))) {
				perms.push('delete');
			}

			return perms;
		}

		getPrimaryKey(cap: Capability) {
			return 'version';
		}

		async viewAll() {
			let res = await panelQuery({
				UpdateChangelog: {
					login_token: $panelAuthState?.loginToken || '',
					action: 'ListEntries'
				}
			});

			if (!res.ok) throw new Error(`Failed to fetch changelog entries: ${await res.text()}`);

			let changelogEntries: ChangelogEntry[] = await res.json();

			return changelogEntries;
		}

		async view(key: string, value: string) {
			let changelogEntries = await this.viewAll();

			return changelogEntries.find((ce) => {
				// @ts-ignore
				return ce[key] == value;
			});
		}

		async create(data: Entry<ChangelogEntry>) {
			let res = await panelQuery({
				UpdateChangelog: {
					login_token: $panelAuthState?.loginToken || '',
					action: {
						CreateEntry: {
							version: data.data.version,
							added: data.data.added,
							updated: data.data.updated,
							removed: data.data.removed,
							extra_description: data.data.extra_description,
							prerelease: data.data.prerelease
						}
					}
				}
			});

			if (!res.ok) throw new Error(`Failed to create changelog entry: ${await res.text()}`);
		}

		async update(data: Entry<ChangelogEntry>) {
			let res = await panelQuery({
				UpdateChangelog: {
					login_token: $panelAuthState?.loginToken || '',
					action: {
						UpdateEntry: {
							version: data.data.version,
							added: data.data.added,
							updated: data.data.updated,
							removed: data.data.removed,
							github_html: data.data.github_html,
							extra_description: data.data.extra_description,
							prerelease: data.data.prerelease,
							published: data.data.published
						}
					}
				}
			});

			if (!res.ok) throw new Error(`Failed to update changelog entry: ${await res.text()}`);
		}

		async delete(data: Entry<ChangelogEntry>) {
			let res = await panelQuery({
				UpdateChangelog: {
					login_token: $panelAuthState?.loginToken || '',
					action: {
						DeleteEntry: {
							version: data.data.version
						}
					}
				}
			});

			if (!res.ok) throw new Error(`Failed to delete changelog entry: ${await res.text()}`);
		}

		async viewToTable(data: ChangelogEntry[]) {
			return {
				fields: this.fields,
				data: data?.map((d) => {
					return {
						...d,
						created_at: new Date(d?.created_at)
					};
				})
			};
		}

		async onOpen(cap: Capability, evt: string, data?: ChangelogEntry) {
			logger.info('ChangelogSchema', 'onOpen', { cap, evt, data });
		}

		warningBox(cap: Capability, data: ChangelogEntry, func: () => Promise<boolean>) {
			switch (cap) {
				case 'delete':
					return {
						header: 'Confirm Deletion',
						text: `Are you sure you want to delete changelog entry for version '${data.version}'? This is an irreversible action.`,
						buttonStates: {
							normal: 'Delete Changelog',
							loading: 'Deleting changelog...',
							success: 'Successfully deleted this changelog',
							error: 'Failed to delete changelog'
						},
						onConfirm: func
					};
				default:
					throw new Error(`Unsupported capability for warningBox: ${cap}`);
			}
		}

		constructor() {
			// Freeze all properties on the class
			for (let key of Object.keys(this)) {
				Object.defineProperty(this, key, {
					writable: false,
					configurable: false
				});
			}

			Object.freeze(this);
		}
	}

	let schema: ChangelogSchema | undefined;

	$: {
		schema = new ChangelogSchema();
	}
</script>

{#if schema}
	<View {schema} />
{:else}
	<Loading msg="Internally creating changelog schema..." />
{/if}
