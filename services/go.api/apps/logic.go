package apps

import (
	"errors"
	"fmt"
	"strings"

	"github.com/anti-raid/splashtail/core/go.std/types"
	"github.com/anti-raid/splashtail/services/go.api/state"

	"github.com/bwmarrin/discordgo"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"
)

var ErrNoPersist = errors.New("no persist") // This error should be returned when the app should not be persisted to the database for review

func reviewLogicBanAppeal(d uapi.RouteData, resp types.AppResponse, reason string, approve bool) error {
	if approve {
		// Unban user

		if len(reason) > 384 {
			return errors.New("reason must be less than 384 characters")
		}

		err := state.Discord.GuildBanDelete(
			state.Config.Servers.Main,
			resp.UserID,
			discordgo.WithAuditLogReason("Ban appeal accepted by "+d.Auth.ID+" | "+reason),
		)

		if err != nil {
			return err
		}
	} else {
		// Denial is always possible
		return nil
	}

	return nil
}

func reviewLogicStaff(d uapi.RouteData, resp types.AppResponse, reason string, approve bool) error {
	if approve {
		err := state.Discord.GuildMemberRoleAdd(state.Config.Servers.Main, resp.UserID, state.Config.Roles.AwaitingStaff)

		if err != nil {
			return err
		}

		// DM the user
		dmchan, err := state.Discord.UserChannelCreate(resp.UserID)

		if err != nil {
			return errors.New("could not send DM, please ask the user to accept DMs from server members")
		}

		if len(reason) > 1024 {
			return errors.New("reason must be 1024 characters or less")
		}

		_, err = state.Discord.ChannelMessageSendComplex(dmchan.ID, &discordgo.MessageSend{
			Embeds: []*discordgo.MessageEmbed{
				{
					Title:       "Staff Application Whitelisted",
					Description: "Your staff application has been whitelisted for onboarding! Please ping any manager at #staff-only in our discord server to get started.",
					Color:       0x00ff00,
					Fields: []*discordgo.MessageEmbedField{
						{
							Name:  "Feedback",
							Value: reason,
						},
					},
					Footer: &discordgo.MessageEmbedFooter{
						Text: "Congratulations!",
					},
				},
			},
		})

		if err != nil {
			return errors.New("could not send DM, please ask the user to accept DMs from server members")
		}

		return nil
	} else {
		if strings.HasPrefix(reason, "MANUALLYNOTIFIED ") {
			state.Logger.Info("forcing denial of staff application that was manually notified by a manager", zap.String("userID", resp.UserID))
			return nil
		}

		// Attempt to DM the user on denial
		dmchan, err := state.Discord.UserChannelCreate(resp.UserID)

		if err != nil {
			return fmt.Errorf("could not create DM channel with user, please inform them manually, then deny with reason of 'MANUALLYNOTIFIED <your reason here>': %w", err)
		}

		_, err = state.Discord.ChannelMessageSendComplex(dmchan.ID, &discordgo.MessageSend{
			Embeds: []*discordgo.MessageEmbed{
				{
					Title:       "Staff Application Denied",
					Description: "Unfortunately, we have denied your staff application for Anti Raid. You may reapply later if you wish to",
					Color:       0x00ff00,
					Fields: []*discordgo.MessageEmbedField{
						{
							Name:  "Reason",
							Value: reason,
						},
					},
					Footer: &discordgo.MessageEmbedFooter{
						Text: "Better luck next time?",
					},
				},
			},
		})

		if err != nil {
			return fmt.Errorf("could not send DM, please inform them manually, then deny with reason of 'MANUALLYNOTIFIED <your reason here>': %w", err)
		}

		return nil
	}
}
