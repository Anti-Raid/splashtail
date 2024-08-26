route = """
package {package}

import (
	"net/http"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"

	"github.com/go-chi/chi/v5"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "{name}",
		Description: "{description}",
		Params: []docs.Parameter{
			{
				Name:        "foo",
				In:          "path",
				Description: "The user's ID",
				Required:    true,
				Schema:      docs.IdSchema,
			},
		},
        /* Add Req+Resp here */
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
    /* Enter code here */
}
"""

def get_input(prompt: str, allow_none=True) -> str | None:
    while True:
        try:
            inp = input(prompt)
            if inp == "none" and allow_none:
                return None
            elif not inp:
                continue
            return inp
        except KeyboardInterrupt:
            exit(1)

module = get_input("Module Name (if you are just making a normal endpoint, type 'none'): ")

if module:
    op_name = get_input("Operation ID (e.g. listsinks): ", allow_none=False)
    op_desc = get_input("Operation Description: ", allow_none=False)
else:
    op_name = get_input("Endpoint ID (e.g. audit_logs): ", allow_none=False)
    op_desc = get_input("Operation Description: ", allow_none=False)