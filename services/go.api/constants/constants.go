package constants

const (
	ResourceNotFound    = "{\"message\":\"Slow down, bucko! We couldn't find this resource *anywhere*!\"}"
	EndpointNotFound    = "{\"message\":\"Slow down, bucko! You got the path wrong or something but this endpoint doesn't exist!\"}"
	BadRequest          = "{\"message\":\"Slow down, bucko! You're doing something illegal!!!\"}"
	Forbidden           = "{\"message\":\"Slow down, bucko! You're not allowed to do this!\"}"
	Unauthorized        = "{\"message\":\"Slow down, bucko! You're not authorized to do this or did you forget a API token somewhere?\"}"
	InternalServerError = "{\"message\":\"Slow down, bucko! Something went wrong on our end!\"}"
	MethodNotAllowed    = "{\"message\":\"Slow down, bucko! That method is not allowed for this endpoint!!!\"}"
	BodyRequired        = "{\"message\":\"Slow down, bucko! A body is required for this endpoint!!!\"}"
	BackTick            = "`"
	DoubleBackTick      = "``"
)
