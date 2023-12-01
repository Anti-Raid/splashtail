import cookie from 'cookie';
import type { Handle } from '@sveltejs/kit';
import type { GetSession } from '@sveltejs/kit';
import * as logger from './lib/logger';

export const handle: Handle = async ({ event, resolve }) => {
  const cookies = cookie.parse(event.request.headers.get('cookie') || '');

  const response = await resolve(event);

  return response;
};

export const getSession: GetSession = async (event) => {
  const cookies = cookie.parse(
    event.request.headers.get('cookie') || event.request.headers.get('Cookie') || ''
  );

  let id = ''
  let instanceUrl = ''

  if (cookies['session']) {
    id = cookies['session'];
    instanceUrl = cookies["instanceUrl"]
  }

  if(!instanceUrl) {
    id = ""
  }

  let maint = false

  if(id) {
    let pingText = ""

    try {
      let pingCheck = await fetch(`${instanceUrl}/ping`, {
        headers: {
          'X-Session': id
        }
      })

      pingText = await pingCheck.text()
    } catch (err) {
      maint = true
      console.error(err)
    }

    if(pingText != "pong") {
      id = ""
    }
  }

  return {id: id, instanceUrl: instanceUrl, maint: maint};
};
