package web

import (
	"encoding/json"
	"fmt"
	"io"
	"mewld/config"
	"mewld/coreutils"
	"mewld/proc"
	"mewld/redis"
	"net/http"
	"net/url"
	"os"
	"strconv"
	"strings"
	"time"

	log "github.com/sirupsen/logrus"

	"github.com/gin-gonic/gin"
)

type SessionStartLimit struct {
	Total          uint64 `json:"total"`
	Remaining      uint64 `json:"remaining"`
	ResetAfter     uint64 `json:"reset_after"`
	MaxConcurrency uint64 `json:"max_concurrency"`
}

type ShardCount struct {
	Shards            uint64            `json:"shards"`
	SessionStartLimit SessionStartLimit `json:"session_start_limit"`
}

func GetShardCount(config config.CoreConfig) ShardCount {
	url := "https://discord.com/api/gateway/bot"

	req, err := http.NewRequest("GET", url, nil)

	req.Header.Add("Authorization", "Bot "+config.Token)
	req.Header.Add("User-Agent", "DiscordBot Antiraid/6.0 (mewld)") // ANTIRAID-SPECIFIC: Change user agent
	req.Header.Add("Content-Type", "application/json")

	if err != nil {
		log.Fatal(err)
	}

	client := http.Client{Timeout: 10 * time.Second}

	res, err := client.Do(req)

	if err != nil {
		log.Fatal(err)
	}

	defer res.Body.Close()

	log.Println("Shard count status:", res.Status)

	if res.StatusCode != 200 {
		log.Fatal("Shard count status code not 200. Invalid token?")
	}

	var shardCount ShardCount

	bodyBytes, err := io.ReadAll(res.Body)

	if err != nil {
		log.Fatal(err)
	}

	err = json.Unmarshal(bodyBytes, &shardCount)

	if err != nil {
		log.Fatal(err)
	}

	if shardCount.SessionStartLimit.Remaining < 10 {
		log.Fatal("Shard count remaining is less than safe value of 10")
	}

	return shardCount
}

type WebData struct {
	RedisHandler *redis.RedisHandler
	InstanceList *proc.InstanceList
}

func checkAuth(webData WebData, c *gin.Context) *loginDat {
	// Get 'session' cookie

	var session string
	var err error

	if c.GetHeader("X-Session") != "" {
		session = c.GetHeader("X-Session")
	} else {
		session, err = c.Cookie("session")
	}

	if err != nil {
		return nil
	}

	// Check session on redis
	redisDat := webData.InstanceList.Redis.Get(webData.InstanceList.Ctx, session).Val()

	if redisDat == "" {
		return nil
	}

	var sess loginDat

	err = json.Unmarshal([]byte(redisDat), &sess)

	if err != nil {
		return nil
	}

	var allowed bool
	for _, id := range webData.InstanceList.Config.AllowedIDS {
		if sess.ID == id {
			allowed = true
			break
		}
	}

	if !allowed {
		log.Error("User not allowed")
		return nil
	}

	return &sess
}

func loginRoute(webData WebData, f func(c *gin.Context, sess *loginDat)) func(c *gin.Context) {
	return func(c *gin.Context) {
		session := checkAuth(webData, c)

		if session != nil {
			f(c, session)
			return
		}

		c.Redirect(302, "/login?redirect="+c.Request.URL.Path)
	}
}

type tokenResponse struct {
	AccessToken string `json:"access_token"`
}

type user struct {
	ID string `json:"id"`
}

type loginDat struct {
	ID          string `json:"id"`
	AccessToken string `json:"access_token"`
}

func StartWebserver(webData WebData) {
	// Create webserver using gin
	r := gin.New()

	r.Use(gin.LoggerWithFormatter(func(param gin.LogFormatterParams) string {
		if param.Path == "/ping" || param.Path == "/action-logs" {
			return ""
		}

		// your custom format
		return fmt.Sprintf("%s - \"%s %s %d %s \"%s\" %s\"\n",
			param.ClientIP,
			param.Method,
			param.Path,
			param.StatusCode,
			param.Latency,
			param.Request.UserAgent(),
			param.ErrorMessage,
		)
	}))
	r.Use(gin.Recovery())
	r.Use(cors())

	// Wildcat route
	r.OPTIONS("/*c", func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", c.GetHeader("Origin"))
		c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, GET, OPTIONS, PUT, PATCH, DELETE")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Accept, Content-Type, Content-Length, Accept-Encoding, X-Session")
		c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
	})

	r.GET("/", func(c *gin.Context) {
		c.String(200, "Mewld instance up, use mewld-ui to access it using your browser")
	})

	r.GET("/ping", loginRoute(
		webData,
		func(c *gin.Context, sess *loginDat) {
			c.String(200, "pong")
		},
	))

	r.GET("/instance-list", loginRoute(
		webData,
		func(c *gin.Context, sess *loginDat) {
			c.JSON(200, webData.InstanceList)
		},
	))

	r.GET("/action-logs", loginRoute(
		webData,
		func(c *gin.Context, sess *loginDat) {
			payload := webData.InstanceList.Redis.Get(webData.InstanceList.Ctx, webData.InstanceList.Config.RedisChannel+"_action").Val()

			c.Header("Content-Type", "text/json")

			c.String(200, payload)
		},
	))

	r.POST("/redis/pub", loginRoute(
		webData,
		func(c *gin.Context, sess *loginDat) {
			payload, err := io.ReadAll(c.Request.Body)

			if err != nil {
				log.Error(err)
				c.String(500, "Error reading body")
				return
			}

			webData.InstanceList.Redis.Publish(webData.InstanceList.Ctx, webData.InstanceList.Config.RedisChannel, string(payload))
		},
	))

	r.GET("/cluster-health", loginRoute(
		webData,
		func(c *gin.Context, sess *loginDat) {
			var cid = c.Query("cid")

			if cid == "" {
				cid = "0"
			}

			cInt, err := strconv.Atoi(cid)

			if err != nil {
				c.JSON(400, gin.H{
					"error": "Invalid cid",
				})
				return
			}

			instance := webData.InstanceList.InstanceByID(cInt)

			if instance == nil {
				c.JSON(400, gin.H{
					"error": "Invalid cid",
				})
				return
			}

			if instance.ClusterHealth == nil {
				if !instance.LaunchedFully {
					c.JSON(400, gin.H{
						"error": "Instance not fully up",
					})
					return
				}
				ch, err := webData.InstanceList.ScanShards(instance)

				if err != nil {
					c.JSON(400, gin.H{
						"error": "Error scanning shards: " + err.Error(),
					})
					return
				}

				instance.ClusterHealth = ch
			}

			c.JSON(200, map[string]any{
				"locked": instance.Locked(),
				"health": instance.ClusterHealth,
			})
		},
	))

	r.GET("/cperms", loginRoute(
		webData,
		func(c *gin.Context, sess *loginDat) {
			// /applications/{application.id}/guilds/{guild.id}/commands/{command.id}/permissions

			guildId := c.Query("guildId")

			if guildId == "" {
				c.JSON(400, gin.H{
					"message": "guildId is required",
				})
				return
			}

			commandId := c.Query("commandId")

			if commandId == "" {
				c.JSON(400, gin.H{
					"message": "commandId is required",
				})
				return
			}

			res, err := http.NewRequest(
				"GET",
				"https://discord.com/api/v10/applications/"+webData.InstanceList.Config.Oauth.ClientID+"/guilds/"+guildId+"/commands/"+commandId+"/permissions",
				nil,
			)

			if err != nil {
				log.Error(err)
				c.String(500, err.Error())
				return
			}

			res.Header.Set("Authorization", "Bot "+os.Getenv("MTOKEN"))
			res.Header.Set("User-Agent", "DiscordBot (WebUI/1.0)")

			client := &http.Client{Timeout: time.Second * 10}

			resp, err := client.Do(res)

			if err != nil {
				log.Error(err)
				c.String(500, err.Error())
				return
			}

			defer resp.Body.Close()

			body, err := io.ReadAll(resp.Body)

			if err != nil {
				log.Error(err)
				c.String(500, err.Error())
				return
			}

			c.Header("Content-Type", "application/json")
			c.String(200, string(body))
		},
	))

	r.GET("/commands", loginRoute(
		webData,
		func(c *gin.Context, sess *loginDat) {
			//"/applications/{application.id}/guilds/{guild.id}/commands"

			guildId := c.Query("guildId")

			if guildId == "" {
				c.JSON(400, gin.H{
					"message": "guildId is required",
				})
				return
			}

			res, err := http.NewRequest(
				"GET",
				"https://discord.com/api/v10/applications/"+webData.InstanceList.Config.Oauth.ClientID+"/guilds/"+guildId+"/commands",
				nil,
			)

			if err != nil {
				log.Error(err)
				c.String(500, err.Error())
				return
			}

			res.Header.Set("Authorization", "Bot "+os.Getenv("MTOKEN"))
			res.Header.Set("User-Agent", "DiscordBot (WebUI/1.0)")

			client := &http.Client{Timeout: time.Second * 10}

			resp, err := client.Do(res)

			if err != nil {
				log.Error(err)
				c.String(500, err.Error())
				return
			}

			defer resp.Body.Close()

			body, err := io.ReadAll(resp.Body)

			if err != nil {
				log.Error(err)
				c.String(500, err.Error())
				return
			}

			c.Header("Content-Type", "application/json")
			c.String(200, string(body))
		},
	))

	r.GET("/guilds", loginRoute(
		webData,
		func(c *gin.Context, sess *loginDat) {
			// Check for guilds on redis
			redisGuilds := webData.InstanceList.Redis.Get(webData.InstanceList.Ctx, sess.ID+"_guilds").Val()

			if redisGuilds != "" {
				c.Header("Content-Type", "application/json")
				c.Header("X-Cached", "true")
				c.String(200, redisGuilds)
				return
			}

			res, err := http.NewRequest("GET", "https://discord.com/api/v10/users/@me/guilds", nil)

			if err != nil {
				log.Error(err)
				c.String(500, err.Error())
				return
			}

			res.Header.Set("Authorization", "Bearer "+sess.AccessToken)

			client := &http.Client{Timeout: time.Second * 10}

			resp, err := client.Do(res)

			if err != nil {
				log.Error(err)
				c.String(500, err.Error())
				return
			}

			defer resp.Body.Close()

			body, err := io.ReadAll(resp.Body)

			if err != nil {
				log.Error(err)
				c.String(500, err.Error())
				return
			}

			webData.InstanceList.Redis.Set(webData.InstanceList.Ctx, sess.ID+"_guilds", string(body), 5*time.Minute).Val()

			c.Header("Content-Type", "application/json")
			c.String(200, string(body))
		},
	))

	r.GET("/login", func(c *gin.Context) {
		// Redirect via discord oauth2
		url := "https://discord.com/api/oauth2/authorize?client_id=" + webData.InstanceList.Config.Oauth.ClientID + "&redirect_uri=" + webData.InstanceList.Config.Oauth.RedirectURL + "/confirm&response_type=code&scope=identify%20guilds%20applications.commands.permissions.update&state=" + c.Query("api")

		// For upcoming sveltekit webui rewrite
		if c.Query("api") == "" {
			c.Redirect(302, url)
		} else {
			c.String(200, url)
		}
	})

	r.GET("/confirm", func(c *gin.Context) {
		// Handle confirmation from discord oauth2
		code := c.Query("code")

		state := c.Query("state")

		// Add form data
		form := url.Values{}
		form["client_id"] = []string{webData.InstanceList.Config.Oauth.ClientID}
		form["client_secret"] = []string{webData.InstanceList.Config.Oauth.ClientSecret}
		form["grant_type"] = []string{"authorization_code"}
		form["code"] = []string{code}
		form["redirect_uri"] = []string{webData.InstanceList.Config.Oauth.RedirectURL + "/confirm"}

		req, err := http.NewRequest("POST", "https://discord.com/api/oauth2/token", strings.NewReader(form.Encode()))

		if err != nil {
			log.Error(err)
			c.String(http.StatusInternalServerError, err.Error())
			return
		}

		// Set headers
		req.Header.Add("User-Agent", "Mewld-webui/1.0")
		req.Header.Add("Content-Type", "application/x-www-form-urlencoded")

		// Create client
		client := http.Client{Timeout: 10 * time.Second}

		// Do request
		res, err := client.Do(req)

		if err != nil {
			log.Error(err)
			c.String(http.StatusInternalServerError, err.Error())
			return
		}

		// Read response
		bodyBytes, err := io.ReadAll(res.Body)

		log.Info(string(bodyBytes))

		if err != nil {
			log.Error(err)
			c.String(http.StatusInternalServerError, err.Error())
			return
		}

		// Parse response
		var discordToken tokenResponse

		err = json.Unmarshal(bodyBytes, &discordToken)

		if err != nil {
			log.Error(err)
			c.String(http.StatusInternalServerError, err.Error())
			return
		}

		// Close body
		res.Body.Close()

		log.Info("Access Token: ", discordToken.AccessToken)

		// Get user info and create session cookie
		req, err = http.NewRequest("GET", "https://discord.com/api/users/@me", nil)

		if err != nil {
			log.Error(err)
			c.String(http.StatusInternalServerError, err.Error())
			return
		}

		// Set headers
		req.Header.Add("User-Agent", "Mewld-webui/1.0")
		req.Header.Add("Authorization", "Bearer "+discordToken.AccessToken)

		// Do request
		res, err = client.Do(req)

		if err != nil {
			log.Error(err)
			c.String(http.StatusInternalServerError, err.Error())
			return
		}

		// Read response
		bodyBytes, err = io.ReadAll(res.Body)

		if err != nil {
			log.Error(err)
			c.String(http.StatusInternalServerError, err.Error())
			return
		}

		// Parse response
		var discordUser user

		err = json.Unmarshal(bodyBytes, &discordUser)

		if err != nil {
			log.Error(err)
			c.String(http.StatusInternalServerError, err.Error())
			return
		}

		log.Info("User Data: ", discordUser)

		var allowed bool
		for _, id := range webData.InstanceList.Config.AllowedIDS {
			if discordUser.ID == id {
				allowed = true
				break
			}
		}

		if !allowed {
			log.Error("User not allowed")
			c.String(http.StatusInternalServerError, "User not allowed")
			return
		}

		sessionTok := coreutils.RandomString(64)

		jsonStruct := loginDat{
			ID:          discordUser.ID,
			AccessToken: discordToken.AccessToken,
		}

		jsonBytes, err := json.Marshal(jsonStruct)

		if err != nil {
			log.Error(err)
			c.String(http.StatusInternalServerError, err.Error())
			return
		}

		webData.InstanceList.Redis.Set(webData.InstanceList.Ctx, sessionTok, string(jsonBytes), time.Minute*30)

		if strings.HasPrefix(state, "api") {
			split := strings.Split(state, "@")

			if len(split) != 3 {
				log.Error("Invalid state")
				c.String(http.StatusInternalServerError, "Invalid state")
				return
			}

			url := split[1]
			iUrl := split[2]

			c.Redirect(302, url+"/ss?session="+sessionTok+"&instanceUrl="+iUrl)
		}

		// Set cookie
		c.SetCookie("session", sessionTok, int(time.Hour.Seconds()), "/", "", false, true)

		// Redirect to dashboard
		c.Redirect(302, "/")
	})

	err := r.Run("127.0.0.1:3820") // listen and serve

	// ANTIRAID-SPECIFIC: Change port and check err returned by r.Run
	if err != nil {
		panic(err)
	}
}
