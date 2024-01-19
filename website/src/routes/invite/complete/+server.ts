import { error, redirect } from '@sveltejs/kit';

/** @type {import('./$types').RequestHandler} */
export async function GET({ request, cookies, url }) {
	let guild_id: String = '';

	const state = url.searchParams.get('state') || null;
	if (!state) throw error(400, 'No state was passed with this request.');

	const extraData = JSON.parse(state);
	if (!extraData) throw error(400, 'Failed to access provided state.');
	else guild_id = extraData.guild_id;

	throw redirect(307, `/invite/${guild_id}?complete_invite=true`);
}
