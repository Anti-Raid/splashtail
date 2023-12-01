package web

import "github.com/gin-gonic/gin"

// Simple allow-all cors middleware

func cors() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", c.GetHeader("Origin"))
		c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, GET, OPTIONS, PUT, PATCH, DELETE")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Accept, Content-Type, Content-Length, Accept-Encoding, X-Session")
		c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
	}
}
