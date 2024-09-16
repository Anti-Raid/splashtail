package types

import (
	"errors"
)

// API configuration data
type ApiConfig struct {
	MainServer          string `json:"main_server" description:"The ID of the main Anti-Raid Discord Server" validate:"required"`
	SupportServerInvite string `json:"support_server_invite" comment:"Discord Support Server Link" default:"https://discord.gg/u78NFAXm" validate:"required"`
	ClientID            string `json:"client_id" description:"The ID of the Anti-Raid bot client" validate:"required"`
}

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
