import { error, redirect } from '@sveltejs/kit';

/** @type {import('./$types').RequestHandler} */
export async function GET({ request, cookies, url }) {
	cookies.set('token', '', {
		path: '/',
		secure: true,
		expires: new Date()
	});

	throw redirect(307, '/');
}
