package get_user_sessions

import (
	"errors"
	"net/http"
	"strings"

	"github.com/anti-raid/splashtail/core/go.std/structparser/db"
	"github.com/anti-raid/splashtail/core/go.std/types"
	"github.com/anti-raid/splashtail/services/go.api/state"
	"github.com/jackc/pgx/v5"
	"go.uber.org/zap"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
)

var (
	webApiTokensCols = strings.Join(db.GetCols(types.UserSession{}), ", ")
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Get User Sessions",
		Description: "Gets all session tokens of a user",
		Resp:        types.UserSessionList{},
		Params:      []docs.Parameter{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	rows, err := state.Pool.Query(d.Context, "SELECT "+webApiTokensCols+" FROM web_api_tokens WHERE user_id = $1", d.Auth.ID)

	if err != nil {
		state.Logger.Error("Error while getting user tokens", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	defer rows.Close()

	tokens, err := pgx.CollectRows(rows, pgx.RowToAddrOfStructByName[types.UserSession])

	if errors.Is(err, pgx.ErrNoRows) {
		return uapi.HttpResponse{
			Status: http.StatusNotFound,
			Json:   types.ApiError{Message: "No sessions found"},
		}
	}

	if err != nil {
		state.Logger.Error("Error while getting user sessions", zap.Error(err))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	return uapi.HttpResponse{
		Json: types.UserSessionList{Sessions: tokens},
	}
}
