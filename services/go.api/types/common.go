package types

import (
	"errors"
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

type ApiErrorWith[T any] struct {
	Data    *T                `json:"data" description:"Any data the client should know about despite the error"`
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

// A clearable is a value that can be either cleared or set
type Clearable[T any] struct {
	Clear bool `json:"clear" description:"Whether or not to clear the value"`
	Value *T   `json:"value" description:"The value to set. Note that clear must be false for this to be used"`
}

// Checks a Clearable for errors
func (c *Clearable[T]) Get() (*T, bool, error) {
	if c.Clear && c.Value != nil {
		return nil, false, errors.New("cannot clear and set a value at the same time")
	}
	if !c.Clear && c.Value == nil {
		return nil, false, errors.New("value must be set if clear is false")
	}
	return c.Value, c.Clear, nil
}
