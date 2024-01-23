<script lang="ts">
	import { panelQuery } from '$lib/fetch';
	import { panelAuthState } from '$lib/panelAuthState';
	import { panelState } from '$lib/panelState';
	import Loading from '../../../components/Loading.svelte';
	import type { Partner } from '$lib/generated/arcadia/Partner';
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
	import type { Partners } from '$lib/generated/arcadia/Partners';
	import type { PartnerType } from '$lib/generated/arcadia/PartnerType';
	import type { CdnAssetItem } from '$lib/generated/arcadia/CdnAssetItem';
	import { convertImage, renderPreview, uploadFileChunks } from '$lib/fileutils';
	import { build, hasPerm } from '$lib/perms';

	/* 
export interface Partner { 
    id: string, 
    name: string, 
    short: string, 
    links: Array<Link>, 
    type: string, 
    created_at: string, 
    user_id: string, 
}
	*/

	class PartnerSchema implements BaseSchema<Partner>, Schema<Partner> {
		name: string = 'partner';
		fields: FieldFetch<Partner> = [
			{
				id: 'id',
				label: 'ID',
				type: 'text',
				helpText: 'The ID of the partner',
				required: true,
				disabled: false,
				renderMethod: 'text'
			},
			{
				id: 'name',
				label: 'Name',
				type: 'text',
				helpText: 'The name of the partner',
				required: true,
				disabled: false,
				renderMethod: 'text'
			},
			{
				id: 'avatar',
				label: 'Avatar',
				type: 'file',
				helpText: 'The avatar of the partner',
				required: true,
				disabled: false,
				renderMethod: 'custom[html]',
				fileUploadData: {
					acceptableMimeTypes: ['image/png', 'image/jpeg', 'image/gif', 'image/webp'],
					renderPreview: async (cap, file, box) => {
						if (!this.mainScope) {
							await this.fetchCdnMainScope();
						}

						return await renderPreview(
							async (_, __) => {
								return file;
							},
							this.mainScope,
							{
								name: `${Date.now()}.webp`,
								path: 'partners',
								size: BigInt(0),
								last_modified: BigInt(0),
								permissions: 0o644,
								is_dir: false
							},
							box
						);
					}
				},
				customRenderer: async (cap: Capability, data: any) => {
					switch (cap) {
						case 'view':
							return `<img style="border-radius: 50%;" width="50px" src="${$panelState?.core_constants?.cdn_url}/avatars/partners/${data.id}.webp" />`;
						default:
							return `${$panelState?.core_constants?.cdn_url}/avatars/partners/${data.id}.webp`;
					}
				}
			},
			{
				id: 'short',
				label: 'Short',
				type: 'textarea', // Its technically short description but longer makes it better/easier
				helpText: 'The short description of the partner',
				required: true,
				disabled: false,
				renderMethod: 'text'
			},
			{
				id: 'links',
				label: 'Links',
				arrayLabel: 'Links',
				type: 'ibl:link',
				helpText: 'The links of the partner',
				required: true,
				disabled: false,
				renderMethod: 'unordered-list'
			},
			async (_) => {
				if (!this.partnerTypes) {
					await this.viewAll(); // Calling this will set the partner types
				}

				return {
					id: 'type',
					label: 'Type',
					type: 'text[choice]',
					selectMenuChoices: this.partnerTypes.map((t) => {
						return {
							value: t.id,
							id: t.id,
							label: t.name
						};
					}),
					helpText: 'The type of the partner',
					required: true,
					disabled: false,
					renderMethod: 'text'
				};
			},
			{
				id: 'created_at',
				label: 'Created At',
				type: 'text',
				helpText: 'The date the partner was created',
				required: false,
				disabled: true,
				renderMethod: 'text'
			},
			{
				id: 'user_id',
				label: 'User ID',
				type: 'text',
				helpText: 'The user ID of the partner',
				required: true,
				disabled: false,
				renderMethod: 'text'
			}
		];

		strictSchemaValidation: boolean = true;
		strictSchemaValidationIgnore: string[] = ['avatar'];

		getCaps(): Capability[] {
			let perms: Capability[] = ['view']; // All staff can view partners
			if (hasPerm($panelState?.staff_member?.resolved_perms || [], build('partners', 'create'))) {
				perms.push('create');
			}
			if (hasPerm($panelState?.staff_member?.resolved_perms || [], build('partners', 'update'))) {
				perms.push('update');
			}
			if (hasPerm($panelState?.staff_member?.resolved_perms || [], build('partners', 'delete'))) {
				perms.push('delete');
			}

			return perms;
		}

		getPrimaryKey(cap: Capability) {
			return 'id';
		}

		async viewAll() {
			let res = await panelQuery({
				UpdatePartners: {
					login_token: $panelAuthState?.loginToken || '',
					action: 'List'
				}
			});

			if (!res.ok) throw new Error(`Failed to fetch partner list: ${await res.text()}`);

			let partners: Partners = await res.json();

			this.partnerTypes = partners.partner_types;
			this.partnerIds = partners.partners.map((p) => p.id);

			return partners.partners;
		}

		async view(key: string, value: string) {
			let entries = await this.viewAll();

			return entries.find((e) => {
				// @ts-ignore
				return e[key] == value;
			});
		}

		async create(data: Entry<Partner>) {
			await this.parseEdit('create', data);
			let res = await panelQuery({
				UpdatePartners: {
					login_token: $panelAuthState?.loginToken || '',
					action: {
						Create: {
							partner: data.data
						}
					}
				}
			});

			if (!res.ok) throw new Error(`Failed to create partner: ${await res.text()}`);
		}

		async update(data: Entry<Partner>) {
			await this.parseEdit('update', data);
			let res = await panelQuery({
				UpdatePartners: {
					login_token: $panelAuthState?.loginToken || '',
					action: {
						Update: {
							partner: data.data
						}
					}
				}
			});

			if (!res.ok) throw new Error(`Failed to update partner: ${await res.text()}`);
		}

		async delete(data: Entry<Partner>) {
			let res = await panelQuery({
				UpdatePartners: {
					login_token: $panelAuthState?.loginToken || '',
					action: {
						Delete: {
							id: data.data.id
						}
					}
				}
			});

			if (!res.ok) throw new Error(`Failed to delete partner: ${await res.text()}`);
		}

		async viewToTable(data: Partner[]) {
			return {
				fields: this.fields,
				data: data?.map((d) => {
					return {
						...d,
						created_at: new Date(d.created_at)
					};
				})
			};
		}

		async onOpen(cap: Capability, evt: string, data?: Partner) {
			logger.info('PartnerSchema', 'onOpen', { cap, evt, data });
		}

		warningBox(cap: Capability, data: Partner, func: () => Promise<boolean>) {
			switch (cap) {
				case 'delete':
					return {
						header: 'Confirm Deletion',
						text: `Are you sure you want to delete partner '${data.id}' ('${data.name}')? This is an irreversible action.`,
						buttonStates: {
							normal: 'Delete Partner',
							loading: 'Deleting partner...',
							success: 'Successfully deleted this partner',
							error: 'Failed to delete partner'
						},
						onConfirm: func
					};
				default:
					throw new Error(`Unsupported capability for warningBox: ${cap}`);
			}
		}

		constructor() {
			// Do nothing
		}

		// Not part of admin panel def
		private partnerTypes: PartnerType[] = [];
		private partnerIds: string[] = [];
		private mainScope: string = '';

		private async fetchCdnMainScope() {
			let scopeRes = await panelQuery({
				GetMainCdnScope: {
					login_token: $panelAuthState?.loginToken || ''
				}
			});

			if (!scopeRes.ok) {
				let err = await scopeRes.text();
				throw new Error(`Failed to fetch main CDN scope: ${err}`);
			}

			this.mainScope = await scopeRes.text();
		}

		private async parseEdit(cap: Capability, entry: Entry<Partner>) {
			if (cap == 'create') {
				if (this.partnerIds.includes(entry.data.id)) {
					throw new Error('Partner ID already exists');
				}

				if (!entry?.files?.['avatar']) {
					throw new Error('No files were uploaded for avatar');
				}
			}

			if (!this.mainScope) {
				await this.fetchCdnMainScope();
			}

			if (!this.partnerIds.length || !this.partnerTypes.length) {
				throw new Error('Partner schema has not been initialized yet');
			}

			entry?.addStatus('Checking links...');

			for (let link of entry.data.links) {
				if (!link.name || !link.value) {
					throw new Error(`Link name or value is empty: ${link.name} ${link.value}`);
				}

				if (!link.value.startsWith('https://')) {
					throw new Error(`Link value must start with https://: ${link.name} ${link.value}`);
				}
			}

			if (entry?.files?.['avatar']) {
				entry?.addStatus('Checking image...');

				let image = entry?.files['avatar'];

				if (!image.type?.startsWith('image/')) {
					throw new Error('Invalid image mime type');
				}

				entry?.addStatus('Checking existing partner image list...');

				let files = await panelQuery({
					UpdateCdnAsset: {
						login_token: $panelAuthState?.loginToken || '',
						path: 'avatars/partners',
						name: '',
						action: 'ListPath',
						cdn_scope: this.mainScope
					}
				});

				if (!files.ok) {
					let err = await files.text();
					throw new Error(`Failed to list CDN path: ${err}`);
				}

				let filesJson: CdnAssetItem[] = await files.json();

				logger.info('PartnerSchema.parseEdit', 'Got CDN files', filesJson);

				let paths = filesJson.map((f) => f.name);

				entry?.addStatus(`=> Found existing images in path: ${paths}`);

				for (let path of paths) {
					if (path == `${entry.data.id}.webp`) {
						entry?.addStatus(`=> Deleting existing partner image: ${path}`);

						let del = await panelQuery({
							UpdateCdnAsset: {
								login_token: $panelAuthState?.loginToken || '',
								path: 'avatars/partners',
								name: path,
								action: 'Delete',
								cdn_scope: this.mainScope
							}
						});

						if (!del.ok) {
							let err = await del.text();
							throw new Error(`Failed to delete existing partner image: ${err}`);
						}

						entry?.addStatus(`=> Deleted existing partner image: ${path}`);
					} else if (path?.split('.')?.length != 2 || !path.endsWith('.webp')) {
						entry?.addStatus(`=> Deleting unknown file: ${path}`);

						let del = await panelQuery({
							UpdateCdnAsset: {
								login_token: $panelAuthState?.loginToken || '',
								path: 'avatars/partners',
								name: path,
								action: 'Delete',
								cdn_scope: this.mainScope
							}
						});

						if (!del.ok) {
							let err = await del.text();
							throw new Error(`Failed to delete unknown file: ${err}`);
						}

						entry?.addStatus(`=> Deleted unknown file: ${path}`);
					}
				}

				// Convert image to webp
				entry?.addStatus('=> Converting image to webp...');

				let webp = await convertImage(image, 'webp');

				// Calculate sha512 hash of the image
				entry?.addStatus('=> Calculating image hash...');
				let hash = await crypto.subtle.digest('sha-512', await webp.arrayBuffer());

				// Convert hash to hex
				let hashArray = Array.from(new Uint8Array(hash));
				let hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');

				entry?.addStatus(`=> Calculated image hash: ${hashHex}`);

				entry?.addStatus('=> Uploading image chunks to CDN...');

				let chunkIds = await uploadFileChunks(webp, {
					onChunkUploaded: (chunkId, size) => {
						entry?.addStatus(`=> Uploaded chunk ${chunkId} (${size} bytes)`);
					}
				});

				entry?.addStatus('=> Creating file with chunk IDs on CDN...');

				let upload = await panelQuery({
					UpdateCdnAsset: {
						login_token: $panelAuthState?.loginToken || '',
						path: 'avatars/partners',
						name: `${entry.data.id}.webp`,
						action: {
							AddFile: {
								overwrite: false,
								chunks: chunkIds,
								sha512: hashHex
							}
						},
						cdn_scope: this.mainScope
					}
				});

				if (!upload.ok) {
					let err = await upload.text();
					throw new Error(`Failed to upload image to CDN: ${err}`);
				}

				entry?.addStatus('=> Uploaded image to CDN');
			}
		}
	}

	let schema: PartnerSchema | undefined;

	$: {
		schema = new PartnerSchema();
	}
</script>

{#if schema}
	<View {schema} />
{:else}
	<Loading msg="Internally creating partner schema..." />
{/if}
