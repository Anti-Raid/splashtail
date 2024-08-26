package types

import (
	"time"

	"github.com/infinitybotlist/eureka/dovewing/dovetypes"
	"github.com/infinitybotlist/eureka/uapi"
)

type Question struct {
	ID          string `json:"id" validate:"required"`
	Question    string `json:"question" validate:"required"`
	Paragraph   string `json:"paragraph" validate:"required"`
	Placeholder string `json:"placeholder" validate:"required"`
	Short       bool   `json:"short" validate:"required"`
}

type Position struct {
	ID        string     `json:"id" validate:"required"`
	Tags      []string   `json:"tags" validate:"required"`
	Info      string     `json:"info" validate:"required"`
	Name      string     `json:"name" validate:"required"`
	Questions []Question `json:"questions" validate:"gt=0,required"`
	Hidden    bool       `json:"hidden"`
	Closed    bool       `json:"closed"`

	// Internal fields
	Channel             func() string                                                                `json:"-"`
	ExtraLogic          func(d uapi.RouteData, p Position, answers map[string]string) error          `json:"-"`
	PositionDescription func(d uapi.RouteData, p Position) string                                    `json:"-"` // Used for custom position descriptions
	AllowedForBanned    bool                                                                         `json:"-"` // If true, banned users can apply for this position
	BannedOnly          bool                                                                         `json:"-"` // If true, only banned users can apply for this position
	ReviewLogic         func(d uapi.RouteData, resp AppResponse, reason string, approved bool) error `json:"-"` // If set, this function will be called when the position is reviewed. If it returns true, the app will be approved/denied
}

type AppMeta struct {
	Positions []Position `json:"positions"`
	Stable    bool       `json:"stable"` // Stable means that the list of apps is not pending big changes
}

type AppResponse struct {
	AppID          string                  `db:"app_id" json:"app_id"`
	User           *dovetypes.PlatformUser `db:"-" json:"user,omitempty"`
	UserID         string                  `db:"user_id" json:"user_id"`
	Questions      []Question              `db:"questions" json:"questions"`
	Answers        map[string]string       `db:"answers" json:"answers"`
	State          string                  `db:"state" json:"state"`
	CreatedAt      time.Time               `db:"created_at" json:"created_at"`
	Position       string                  `db:"position" json:"position"`
	ReviewFeedback *string                 `db:"review_feedback" json:"review_feedback"`
}

type AppListResponse struct {
	Apps []AppResponse `json:"apps"`
}
