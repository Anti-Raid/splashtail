import type { RequestHandler } from "@sveltejs/kit";

export const GET: RequestHandler = async (event) => {
  let sessionId = event.url.searchParams.get("session")
  let instanceUrl = event.url.searchParams.get("instanceUrl")

  return {
    "headers": {
      "set-cookie": [`session=${sessionId}; path=/; HttpOnly; SameSite=Lax;`, `instanceUrl=${instanceUrl}; path=/; HttpOnly; SameSite=Lax;`],
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
