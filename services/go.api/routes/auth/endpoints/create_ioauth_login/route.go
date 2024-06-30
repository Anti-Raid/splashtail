package create_ioauth_login

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
	"slices"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/core/go.std/types"
	"github.com/anti-raid/splashtail/services/go.api/state"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/eureka/crypto"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/ratelimit"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Create IOAuth Login",
		Description: "Creates an IOAuth (Immediate/Implicit oauth) login for in-bot endpoints that need oauth2 authentication.",
		Resp:        types.ApiError{},
		Params: []docs.Parameter{
			{
				Name:        "rd",
				Description: "The redirect data as a base64",
				Required:    true,
				In:          "query",
				Schema:      docs.IdSchema,
			},
			{
				Name:        "code",
				Description: "The code of the data",
				Required:    true,
				In:          "query",
				Schema:      docs.IdSchema,
			},
		},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	limit, err := ratelimit.Ratelimit{
		Expiry:      5 * time.Minute,
		MaxRequests: 15,
		Bucket:      "ioauth",
		Identifier: func(r *http.Request) string {
			return d.Auth.ID
		},
	}.Limit(d.Context, r)

	if err != nil {
		state.Logger.Error("Error while ratelimiting", zap.Error(err), zap.String("bucket", "ioauth"))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	if limit.Exceeded {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "You are being ratelimited. Please try again in " + limit.TimeToReset.String(),
			},
			Headers: limit.Headers(),
			Status:  http.StatusTooManyRequests,
		}
	}

	var rdB string
	// Or state
	if r.URL.Query().Get("code") != "" && r.URL.Query().Get("state") != "" {
		rdB = r.URL.Query().Get("state")
	} else {
		rdB = r.URL.Query().Get("rd")
	}

	if rdB == "" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "rd query parameter is required"},
		}
	}

	// Decode redirect data
	rdDecoded, err := base64.RawURLEncoding.DecodeString(rdB)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "rd query parameter is not a valid base64 string: " + err.Error()},
		}
	}

	var rd types.IOAuthRedirect

	err = json.Unmarshal(rdDecoded, &rd)

	if err != nil {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "rd query parameter is not valid redirect data: " + err.Error()},
		}
	}

	if !strings.HasPrefix(rd.Dest, "/") && rd.Dest != "json" {
		return uapi.HttpResponse{
			Status: http.StatusBadRequest,
			Json:   types.ApiError{Message: "rd query parameter is not valid redirect data: dest must be a relative path"},
		}
	}

	redirectUrl := state.Config.Sites.API.Parse() + "/ioauth/login"

	if r.URL.Query().Get("code") == "" {
		// Redirect user
		return uapi.HttpResponse{
			Status:   http.StatusTemporaryRedirect,
			Redirect: fmt.Sprintf("https://discord.com/api/v10/oauth2/authorize?client_id=%s&redirect_uri=%s&response_type=code&scope=%s&state=%s", state.Config.DiscordAuth.ClientID, redirectUrl, strings.Join(rd.Scopes, "%20"), rdB),
		}
	} else {
		// Check for reuse
		reused, err := state.Rueidis.Do(d.Context, state.Rueidis.B().Exists().Key("ioauth:codereuse:{"+r.URL.Query().Get("code")+"}").Build()).AsInt64()

		if err != nil {
			state.Logger.Error("Error while checking for reuse", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json:   types.ApiError{Message: "Error while checking for reuse: " + err.Error()},
			}
		}

		if reused > 0 {
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json:   types.ApiError{Message: "This code has already been used"},
			}
		}

		var iot types.IOAuthOutput

		// Exchange code for token
		req, err := http.NewRequest("POST", state.Config.Meta.Proxy.Parse()+"/api/v10/oauth2/token", strings.NewReader(url.Values{
			"client_id":     {state.Config.DiscordAuth.ClientID},
			"client_secret": {state.Config.DiscordAuth.ClientSecret},
			"grant_type":    {"authorization_code"},
			"code":          {r.URL.Query().Get("code")},
			"redirect_uri":  {redirectUrl},
		}.Encode()))

		if err != nil {
			state.Logger.Error("Error while creating request", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json:   types.ApiError{Message: "Error while creating request: " + err.Error()},
			}
		}

		req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
		req.Header.Set("User-Agent", "Anti-Raid (splashtail, https://github.com/Anti-Raid/splashtail)")

		client := http.Client{
			Timeout: 5 * time.Second,
		}

		resp, err := client.Do(req)

		if err != nil {
			state.Logger.Error("Error while exchanging code for token", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json:   types.ApiError{Message: "Error while exchanging code for token: " + err.Error()},
			}
		}

		if resp.StatusCode != http.StatusOK {
			var errData map[string]any

			err = json.NewDecoder(resp.Body).Decode(&errData)

			if err != nil {
				state.Logger.Error("Error while decoding error response", zap.Error(err))
				return uapi.HttpResponse{
					Status: http.StatusBadRequest,
					Json:   types.ApiError{Message: "Error while decoding error response: " + err.Error()},
				}
			}

			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json: types.IOAuthDiscordError{
					Context: errData,
					Message: "Error while exchanging code for token",
				},
			}
		}

		// Save to token reuse redis
		err = state.Rueidis.Do(d.Context, state.Rueidis.B().Set().Key("ioauth:codereuse:{"+r.URL.Query().Get("code")+"}").Value("1").Ex(5*time.Minute).Build()).Error()

		if err != nil {
			state.Logger.Error("Error while saving token reuse", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json:   types.ApiError{Message: "Error while saving token reuse: " + err.Error()},
			}
		}

		err = json.NewDecoder(resp.Body).Decode(&iot)

		if err != nil {
			state.Logger.Error("Error while decoding response", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json:   types.ApiError{Message: "Error while decoding response: " + err.Error()},
			}
		}

		iot.Scope = strings.ReplaceAll(iot.Scope, "%20", " ")

		iot.Scopes = strings.Split(iot.Scope, " ")

		/*if len(iot.Scopes) != len(rd.Scopes) {
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json:   types.ApiError{Message: fmt.Sprintf("Invalid scopes. Expected %s, got %s (%s)", strings.Join(rd.Scopes, ", "), strings.Join(iot.Scopes, ", "), iot.Scope)},
			}
		}*/

		// Ensure that all scopes in rd.Scopes are in iot.Scopes
		for _, scope := range rd.Scopes {
			if !slices.Contains(iot.Scopes, scope) {
				return uapi.HttpResponse{
					Status: http.StatusBadRequest,
					Json:   types.ApiError{Message: fmt.Sprintf("Invalid scopes. Expected %s, got %s (missing scope %s)", strings.Join(rd.Scopes, ", "), strings.Join(iot.Scopes, ", "), scope)},
				}
			}
		}

		if !slices.Contains(iot.Scopes, "identify") {
			return uapi.HttpResponse{
				Status: http.StatusBadRequest,
				Json:   types.ApiError{Message: "Invalid scopes. Expected identify scope to be in " + strings.Join(iot.Scopes, ", ")},
			}
		}

		for _, scope := range iot.Scopes {
			switch scope {
			case "identify":
				// Fetch user information
				req, err = http.NewRequest("GET", state.Config.Meta.Proxy.Parse()+"/api/v10/users/@me", nil)

				if err != nil {
					state.Logger.Error("Error while creating request", zap.Error(err))
					return uapi.HttpResponse{
						Status: http.StatusBadRequest,
						Json:   types.ApiError{Message: "Error while creating request: " + err.Error()},
					}
				}

				req.Header.Set("Authorization", "Bearer "+iot.AccessToken)
				req.Header.Set("User-Agent", "Anti-Raid (splashtail, https://github.com/Anti-Raid/splashtail)")

				resp, err = client.Do(req)

				if err != nil {
					state.Logger.Error("Error while fetching user information", zap.Error(err))
					return uapi.HttpResponse{
						Status: http.StatusBadRequest,
						Json:   types.ApiError{Message: "Error while fetching user information: " + err.Error()},
					}
				}

				if resp.StatusCode != http.StatusOK {
					var errData map[string]any

					err = json.NewDecoder(resp.Body).Decode(&errData)

					if err != nil {
						state.Logger.Error("Error while decoding error response", zap.Error(err))
						return uapi.HttpResponse{
							Status: http.StatusBadRequest,
							Json:   types.ApiError{Message: "Error while decoding error response for /users/@me: " + err.Error()},
						}
					}

					return uapi.HttpResponse{
						Status: http.StatusBadRequest,
						Json: types.IOAuthDiscordError{
							Context: errData,
							Message: "Error while fetching user information",
						},
					}
				}

				var user *discordgo.User

				err = json.NewDecoder(resp.Body).Decode(&user)

				if err != nil {
					state.Logger.Error("Error while decoding response", zap.Error(err))
					return uapi.HttpResponse{
						Status: http.StatusBadRequest,
						Json:   types.ApiError{Message: "Error while decoding response for /users/@me: " + err.Error()},
					}
				}

				iot.DiscordUser = user
			case "guilds":
				// Fetch user guilds information
				req, err = http.NewRequest("GET", state.Config.Meta.Proxy.Parse()+"/api/v10/users/@me/guilds", nil)

				if err != nil {
					state.Logger.Error("Error while creating request", zap.Error(err))
					return uapi.HttpResponse{
						Status: http.StatusBadRequest,
						Json:   types.ApiError{Message: "Error while creating request: " + err.Error()},
					}
				}

				req.Header.Set("Authorization", "Bearer "+iot.AccessToken)
				req.Header.Set("User-Agent", "Anti-Raid (splashtail, https://github.com/Anti-Raid/splashtail)")

				resp, err = client.Do(req)

				if err != nil {
					state.Logger.Error("Error while fetching user information", zap.Error(err))
					return uapi.HttpResponse{
						Status: http.StatusBadRequest,
						Json:   types.ApiError{Message: "Error while fetching user information: " + err.Error()},
					}
				}

				if resp.StatusCode != http.StatusOK {
					var errData map[string]any

					err = json.NewDecoder(resp.Body).Decode(&errData)

					if err != nil {
						state.Logger.Error("Error while decoding error response", zap.Error(err))
						return uapi.HttpResponse{
							Status: http.StatusBadRequest,
							Json:   types.ApiError{Message: "Error while decoding error response for /users/@me: " + err.Error()},
						}
					}

					return uapi.HttpResponse{
						Status: http.StatusBadRequest,
						Json: types.IOAuthDiscordError{
							Context: errData,
							Message: "Error while fetching user information",
						},
					}
				}

				var userGuilds []*discordgo.UserGuild

				err = json.NewDecoder(resp.Body).Decode(&userGuilds)

				if err != nil {
					state.Logger.Error("Error while decoding response", zap.Error(err))
					return uapi.HttpResponse{
						Status: http.StatusBadRequest,
						Json:   types.ApiError{Message: "Error while decoding response for /users/@me: " + err.Error()},
					}
				}

				iot.UserGuilds = userGuilds
			}
		}

		// Create ioauth token
		iotB, err := json.Marshal(iot)

		if err != nil {
			state.Logger.Error("Error while marshalling ioauth output", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   types.ApiError{Message: "Error while marshalling ioauth output: " + err.Error()},
			}
		}

		token := crypto.RandString(32)
		err = state.Rueidis.Do(d.Context, state.Rueidis.B().Set().Key("ioauth:{"+token+"}").Value(string(iotB)).Ex(5*time.Minute).Build()).Error()

		if err != nil {
			state.Logger.Error("Error while creating token", zap.Error(err))
			return uapi.HttpResponse{
				Status: http.StatusInternalServerError,
				Json:   types.ApiError{Message: "Error while creating token: " + err.Error()},
			}
		}

		if rd.Dest == "json" {
			return uapi.HttpResponse{
				Json: iot,
			}
		} else {
			return uapi.HttpResponse{
				Status: http.StatusTemporaryRedirect,
				Redirect: func() string {
					if strings.Contains(rd.Dest, "?") {
						return rd.Dest + "&ioauth=" + token
					} else {
						return rd.Dest + "?ioauth=" + token
					}
				}(),
			}
		}
	}
}
