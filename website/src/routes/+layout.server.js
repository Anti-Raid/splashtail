import cookie from 'cookie';

export const load = async ({ request, setHeaders }) => {
	const cookies = cookie.parse(request.headers.get('cookie') || '');

	if (cookies.token) {
		const userData = await fetch(
			`https://api.antiraid.xyz/api/users/getwithtoken?token=${cookies.token}`
		)
			.then((res) => res.json())
			.catch((err) => {
				throw new Error(err);
			});

		if (userData.error)
			return {
				user: null
			};
		else
			return {
				user: userData
			};
	} else
		return {
			user: null
		};
};
