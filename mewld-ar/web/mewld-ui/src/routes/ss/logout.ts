import type { RequestHandler } from "@sveltejs/kit";

export const GET: RequestHandler = async (event) => {
  let sessionId = event.url.searchParams.get("session")

  return {
    "headers": {
      "set-cookie": `session=; path=/; HttpOnly; SameSite=Lax; Expires=Thu, 01 Jan 1970 00:00:00 GMT`,
      "content-type": "text/html"
    },
    "body": `
      <html>
        <body>
          <p>Redirecting to /</p>
          <script>
            window.location.href = "/";
          </script>
        </body>
      </html>
    `
  }
}
