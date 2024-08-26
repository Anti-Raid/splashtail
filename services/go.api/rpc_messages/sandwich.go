package rpc_messages

type SandwichBaseRestResponse[T any] struct {
	Data  *T     `json:"data,omitempty"`
	Error string `json:"error,omitempty"`
	Ok    bool   `json:"ok"`
}
