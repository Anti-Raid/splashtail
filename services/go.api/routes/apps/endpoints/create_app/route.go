package create_app

import (
	"errors"
	"net/http"

	"go.api/apps"
	"go.api/state"
	"go.api/types"

	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"

	"github.com/bwmarrin/discordgo"
	"github.com/go-playground/validator/v10"
	"github.com/infinitybotlist/eureka/crypto"
)

type CreateApp struct {
	Position string            `json:"position" validate:"required"`
	Answers  map[string]string `json:"answers" validate:"required,dive,required"`
}

var compiledMessages = uapi.CompileValidationErrors(CreateApp{})

func Docs() *docs.Doc {
	return &docs.Doc{
		Summary:     "Create App For Position",
		Description: "Creates an application for a position. Returns a 204 on success.",
		Req:         CreateApp{},
		Params:      []docs.Parameter{},
		Resp:        types.ApiError{},
	}
}

func Route(d uapi.RouteData, r *http.Request) uapi.HttpResponse {
	var payload CreateApp

	hresp, ok := uapi.MarshalReq(r, &payload)

	if !ok {
		return hresp
	}

	// Validate the payload

	err := state.Validator.Struct(payload)

	if err != nil {
		errors := err.(validator.ValidationErrors)
		return uapi.ValidatorErrorResponse(compiledMessages, errors)
	}

	position := apps.FindPosition(payload.Position)

	if position == nil {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Invalid position",
			},
			Status: http.StatusBadRequest,
		}
	}

	if d.Auth.Banned && !position.AllowedForBanned {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "Banned users are not allowed to apply for this position",
			},
			Status: http.StatusBadRequest,
		}
	}

	if !d.Auth.Banned && position.BannedOnly {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "You are not banned? Why are you appealing?",
			},
			Status: http.StatusBadRequest,
		}
	}

	if position.Closed {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "This position is currently closed. Please check back later.",
			},
			Status: http.StatusBadRequest,
		}
	}

	var userApps int64

	err = state.Pool.QueryRow(d.Context, "SELECT COUNT(*) FROM apps WHERE user_id = $1 AND position = $2 AND state = 'pending'", d.Auth.ID, payload.Position).Scan(&userApps)

	if err != nil {
		state.Logger.Error("Error getting user apps", zap.Error(err), zap.String("user_id", d.Auth.ID), zap.String("position", payload.Position))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	if userApps > 0 {
		return uapi.HttpResponse{
			Json: types.ApiError{
				Message: "You already have a pending application for this position",
			},
			Status: http.StatusBadRequest,
		}
	}

	var answerMap = map[string]string{}

	for _, question := range position.Questions {
		ans, ok := payload.Answers[question.ID]

		if !ok {
			return uapi.HttpResponse{
				Json: types.ApiError{
					Message: "Missing answer for question " + question.ID,
				},
				Status: http.StatusBadRequest,
			}
		}

		if ans == "" {
			return uapi.HttpResponse{
				Json: types.ApiError{
					Message: "Answer for question " + question.ID + " cannot be empty",
				},
				Status: http.StatusBadRequest,
			}
		}

		if question.Short {
			if len(ans) > 4096 {
				return uapi.HttpResponse{
					Json: types.ApiError{
						Message: "Answer for question " + question.ID + " is too long",
					},
					Status: http.StatusBadRequest,
				}
			}
		} else {
			if len(ans) < 50 {
				return uapi.HttpResponse{
					Json: types.ApiError{
						Message: "Answer for question " + question.ID + " is too short",
					},
					Status: http.StatusBadRequest,
				}
			}

			if len(ans) > 10000 {
				return uapi.HttpResponse{
					Json: types.ApiError{
						Message: "Answer for question " + question.ID + " is too long",
					},
					Status: http.StatusBadRequest,
				}
			}
		}

		answerMap[question.ID] = ans
	}

	var noPersistToDatabase bool
	if position.ExtraLogic != nil {
		err := position.ExtraLogic(d, *position, answerMap)

		if err != nil {
			state.Logger.Error("Error running extra logic", zap.Error(err), zap.String("user_id", d.Auth.ID), zap.String("position", payload.Position))
			return uapi.HttpResponse{
				Json: types.ApiError{
					Message: "Error: " + err.Error(),
				},
				Status: http.StatusBadRequest,
			}
		}

		if errors.Is(err, apps.ErrNoPersist) {
			noPersistToDatabase = true
		}
	}

	var appId string
	if !noPersistToDatabase {
		appId = crypto.RandString(64)

		_, err = state.Pool.Exec(
			d.Context,
			"INSERT INTO apps (app_id, user_id, position, questions, answers) VALUES ($1, $2, $3, $4, $5)",
			appId,
			d.Auth.ID,
			payload.Position,
			position.Questions,
			answerMap,
		)

		if err != nil {
			state.Logger.Error("Error inserting app", zap.Error(err), zap.String("user_id", d.Auth.ID), zap.String("position", payload.Position))
			return uapi.DefaultResponse(http.StatusInternalServerError)
		}
	} else {
		appId = "Not Applicable (not persisted to database)"
	}

	// Send a message to APPS channel
	var desc = "User <@" + d.Auth.ID + "> has applied for " + payload.Position + "."
	if position.PositionDescription != nil {
		desc = position.PositionDescription(d, *position)
	}

	var channel = state.Config.Channels.Apps

	if position.Channel != nil {
		channel = position.Channel()
	}

	_, err = state.Discord.ChannelMessageSendComplex(channel, &discordgo.MessageSend{
		Content: "<@&" + state.Config.Roles.Apps + ">",
		Embeds: []*discordgo.MessageEmbed{
			{
				Title:       "New " + position.Name + " Application!",
				URL:         state.Config.Sites.Panel + "/panel/apps",
				Description: desc,
				Color:       0x00ff00,
				Fields: []*discordgo.MessageEmbedField{
					{
						Name:   "App ID",
						Value:  appId,
						Inline: true,
					},
					{
						Name:   "User ID",
						Value:  d.Auth.ID,
						Inline: true,
					},
					{
						Name:   "Position",
						Value:  payload.Position,
						Inline: true,
					},
				},
			},
		},
	})

	if err != nil {
		state.Logger.Error("Error sending embed to apps channel", zap.Error(err), zap.String("user_id", d.Auth.ID), zap.String("position", payload.Position))
		return uapi.DefaultResponse(http.StatusInternalServerError)
	}

	return uapi.DefaultResponse(http.StatusNoContent)
}
