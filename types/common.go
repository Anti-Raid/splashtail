package types

import (
	"time"

	"github.com/jackc/pgx/v5/pgtype"
)

// A link is any extra link
type Link struct {
	Name  string `json:"name" description:"Name of the link. Links starting with an underscore are 'asset links' and are not visible"`
	Value string `json:"value" description:"Value of the link. Must normally be HTTPS with the exception of 'asset links'"`
}

// SEO object (minified bot/user/server for seo purposes)
type SEO struct {
	Name   string `json:"name" description:"Name of the entity"`
	ID     string `json:"id" description:"ID of the entity"`
	Avatar string `json:"avatar" description:"The entities resolved avatar URL (not just hash)"`
	Short  string `json:"short" description:"Short description of the entity"`
}

// This represents a IBL Popplio API Error
type ApiError struct {
	Context map[string]string `json:"context,omitempty" description:"Context of the error. Usually used for validation error contexts"`
	Message string            `json:"message" description:"Message of the error"`
}

// Paged result common
type PagedResult[T any] struct {
	Count   uint64 `json:"count"`
	PerPage uint64 `json:"per_page"`
	Results T      `json:"results"`
}

type Vanity struct {
	ITag       pgtype.UUID `db:"itag" json:"itag" description:"The vanities internal ID."`
	TargetID   string      `db:"target_id" json:"target_id" description:"The ID of the entity"`
	TargetType string      `db:"target_type" json:"target_type" description:"The type of the entity"`
	Code       string      `db:"code" json:"code" description:"The code of the vanity"`
	CreatedAt  time.Time   `db:"created_at" json:"created_at" description:"The time the vanity was created"`
}
