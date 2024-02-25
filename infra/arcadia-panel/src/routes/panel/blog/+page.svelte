<script lang="ts">
	import { fetchClient, panelQuery } from '$lib/fetch';
	import type { BlogPost } from '$lib/generated/arcadia/BlogPost';
	import { panelState } from '$lib/panelState';
	import { panelAuthState } from '$lib/panelAuthState';
	import { newField } from '../../../components/admin/helpers';
	import type {
		BaseSchema,
		Capability,
		CustomAction,
		CustomActionFetch,
		Entry,
		FieldFetch,
		Schema
	} from '../../../components/admin/types';
	import logger from '$lib/logger';
	import View from '../../../components/admin/View.svelte';
	import Loading from '../../../components/Loading.svelte';
	import { Color } from '../../../components/button/colors';
	import type { Query } from '$lib/generated/htmlsanitize/Query';
	import { build, hasPerm } from '$lib/perms';

	/* 
export interface BlogPost { 
    itag: string, 
    slug: string, 
    title: string, 
    description: string, 
    user_id: string, 
    created_at: string, 
    content: string, 
    draft: boolean, 
    tags: Array<string>, 
}
	*/

	class BlogSchema implements BaseSchema<BlogPost>, Schema<BlogPost> {
		name: string = 'blog';
		fields: FieldFetch<BlogPost> = [
			async (cap) => {
				if (cap == 'create') return null; // itag is not available in create
				return newField('itag', 'ID', 'The ID of the blog post', true, true); // ID is not editable
			},
			newField('slug', 'Slug', 'The slug of the blog post', true, false),
			newField('title', 'Title', 'The title of the blog post', true, false),
			newField('description', 'Description', 'The description of the blog post', true, false),
			async (cap) => {
				if (cap == 'create') return null; // user_id is not available in create
				return newField(
					'user_id',
					'User ID',
					'The ID of the user who created the blog post',
					true,
					true
				); // user_id is not editable
			},
			async (cap) => {
				if (cap == 'create') return null; // created_at is not available in create
				return newField(
					'created_at',
					'Created At',
					'The time the blog post was created at',
					true,
					true
				); // user_id is not editable
			},
			newField('content', 'Content', 'The content of the blog post', true, false, {
				type: 'textarea',
				renderMethod: 'custom[html]',
				customRenderer: async (cap: Capability, data: BlogPost) => {
					return data.content.replaceAll('\n', '<br />').slice(0, 100) + '...';
				}
			}),
			async (cap) => {
				if (cap == 'create') return null; // draft is not available in create
				return newField('draft', 'Draft', 'Whether the blog post is a draft', true, false, {
					type: 'boolean'
				});
			},
			newField('tags', 'Tags', 'The tags of the blog post', true, false, {
				type: 'text[]',
				renderMethod: 'unordered-list'
			})
		];

		strictSchemaValidation: boolean = true;
		strictSchemaValidationIgnore: string[] = [];

		getCaps(): Capability[] {
			let perms: Capability[] = ['view']; // All staff can view partners
			if (hasPerm($panelState?.staff_member?.resolved_perms || [], build('blog', 'create_entry'))) {
				perms.push('create');
			}
			if (hasPerm($panelState?.staff_member?.resolved_perms || [], build('blog', 'update_entry'))) {
				perms.push('update');
			}
			if (hasPerm($panelState?.staff_member?.resolved_perms || [], build('blog', 'delete_entry'))) {
				perms.push('delete');
			}

			return perms;
		}

		getPrimaryKey(cap: Capability) {
			if (cap == 'create') return 'slug';
			return 'itag';
		}

		async viewAll() {
			let res = await panelQuery({
				UpdateBlog: {
					login_token: $panelAuthState?.loginToken || '',
					action: 'ListEntries'
				}
			});

			if (!res.ok) throw new Error(`Failed to fetch blog entries: ${await res.text()}`);

			let blogPosts: BlogPost[] = await res.json();

			return blogPosts;
		}

		async view(key: string, value: string) {
			let blogPost = await this.viewAll();

			return blogPost.find((ce) => {
				// @ts-ignore
				return ce[key] == value;
			});
		}

		async create(data: Entry<BlogPost>) {
			let res = await panelQuery({
				UpdateBlog: {
					login_token: $panelAuthState?.loginToken || '',
					action: {
						CreateEntry: {
							slug: data.data.slug,
							title: data.data.title,
							description: data.data.description,
							content: data.data.content,
							tags: data.data.tags
						}
					}
				}
			});

			if (!res.ok) throw new Error(`Failed to create blog post: ${await res.text()}`);
		}

		async update(data: Entry<BlogPost>) {
			let res = await panelQuery({
				UpdateBlog: {
					login_token: $panelAuthState?.loginToken || '',
					action: {
						UpdateEntry: {
							itag: data.data.itag,
							slug: data.data.slug,
							title: data.data.title,
							description: data.data.description,
							content: data.data.content,
							tags: data.data.tags,
							draft: data.data.draft
						}
					}
				}
			});

			if (!res.ok) throw new Error(`Failed to update blog post: ${await res.text()}`);
		}

		async delete(data: Entry<BlogPost>) {
			let res = await panelQuery({
				UpdateBlog: {
					login_token: $panelAuthState?.loginToken || '',
					action: {
						DeleteEntry: {
							itag: data.data.itag
						}
					}
				}
			});

			if (!res.ok) throw new Error(`Failed to delete blog post: ${await res.text()}`);
		}

		async viewToTable(data: BlogPost[]) {
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

		async onOpen(cap: Capability, evt: string, data?: BlogPost) {
			logger.info('BlogPost', 'onOpen', { cap, evt, data });
		}

		warningBox(cap: Capability, data: BlogPost, func: () => Promise<boolean>) {
			switch (cap) {
				case 'delete':
					return {
						header: 'Confirm Deletion',
						text: `Are you sure you want to delete blog post with slug '${data.slug}'? This is an irreversible action.`,
						buttonStates: {
							normal: 'Delete Blog Post',
							loading: 'Deleting blog post...',
							success: 'Successfully deleted this blog post',
							error: 'Failed to delete blog post'
						},
						onConfirm: func
					};
				default:
					throw new Error(`Unsupported capability for warningBox: ${cap}`);
			}
		}

		customActions: CustomActionFetch<BlogPost> = [
			async (cap: Capability) => {
				return {
					label: 'Preview Post',
					helpText: 'See what the post will look like to a user',
					action: async (cap, data, div) => {
						let content = await this.fetchContentFromHtmlSanitize(data.content);
						div.innerHTML = `
                            <div class="desc">
                                ${content}
                            </div>
                        `;
						return true;
					},
					button: {
						icon: 'mdi:eye',
						color: Color.Themable,
						states: {
							normal: 'Preview Post',
							loading: 'Loading preview...',
							success: 'Successfully loaded preview',
							error: 'Failed to load preview'
						}
					}
				};
			}
		];

		constructor() {
			// Do nothing
		}

		private fetchContentFromHtmlSanitize = async (content: string) => {
			let data: Query = {
				SanitizeRaw: {
					body: content
				}
			};
			let res = await fetchClient(`${$panelState?.core_constants?.htmlsanitize_url}/query`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify(data)
			});

			if (!res.ok) throw new Error(`Failed to sanitize HTML: ${await res.text()}`);

			return await res.text();
		};
	}

	let schema: BlogSchema | undefined;

	$: {
		schema = new BlogSchema();
	}
</script>

{#if schema}
	<View {schema} />
{:else}
	<Loading msg="Internally creating blog schema..." />
{/if}
