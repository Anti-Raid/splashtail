package endpoints

type IPC struct {
	// Description is the description of the IPC event
	Description string

	// Deprecated is whether or not the IPC event is deprecated.
	// If yes, then this field should be the reason why.
	Deprecated string

	// Exec is the function to execute when the IPC event is received
	Exec func(client string, args map[string]any) (map[string]any, error)
}
