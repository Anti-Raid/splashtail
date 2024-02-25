import { panelAuthProtocolVersion } from './constants';
import { panelQuery } from './fetch';
import logger from './logger';
import type { PanelAuthState } from './panelAuthState';
import { sleep } from './time';
import { error } from './toast';

export const logoutUser = async (sendLogout: boolean) => {
	if (sendLogout) {
		try {
			let panelStateData = localStorage.getItem('panelStateData');
			let json: PanelAuthState = JSON.parse(panelStateData || 'null');

			let resp = await panelQuery({
				Authorize: {
					version: panelAuthProtocolVersion,
					action: {
						Logout: {
							login_token: json?.loginToken
						}
					}
				}
			});

			if (!resp.ok) {
				let err = await resp.text();

				throw new Error(err);
			}

			let numSessionsDestroyed = await resp.text();

			logger.info(
				'Panel.Logout',
				`Destroyed ${numSessionsDestroyed} sessions associated with this login token`
			);
		} catch (err) {
			error(err?.toString() || 'Unknown error');

			await sleep(1000);
		}
	}

	localStorage.clear();
	window.location.reload();
};
