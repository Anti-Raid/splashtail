package get_asset

import (
	"errors"
	"io/fs"
	"net/http"

	"github.com/anti-raid/splashtail/data"
	"github.com/anti-raid/splashtail/services/go.api/types"
	"github.com/go-chi/chi/v5"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get Asset",
		Description: "This endpoint returns the content of an asset located with `data` (see splashtail GitHub for more information).",
		Resp:        struct{}{},
		Params: []docs.Parameter{
			{
				Name:        "asset",
				In:          "path",
				Description: "The asset to get.",
				Required:    true,
				Schema:      docs.IdSchema,
			},
		},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	asset := chi.URLParam(r, "asset")

	if asset == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "asset is required"},
		}
	}

	file, err := data.Data.ReadFile(asset)

	if errors.Is(err, fs.ErrNotExist) {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json:   types.ApiError{Message: "Asset not found: " + err.Error()},
		}
	}

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusInternalServerError,
			Json:   types.ApiError{Message: "Failed to read asset: " + err.Error()},
		}
	}

	return uapi.HttpResponse{
		Bytes: file,
	}
}
