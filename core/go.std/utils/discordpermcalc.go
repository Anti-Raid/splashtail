package utils

import (
	"slices"

	"github.com/bwmarrin/discordgo"
)

// Computes the 'base permissions' of a member
func BasePermissions(g *discordgo.Guild, m *discordgo.Member) int64 {
	if g.OwnerID == m.User.ID {
		return discordgo.PermissionAll
	}

	// Get everyone role of the guild
	var everyoneRole *discordgo.Role

	var roleMap = make(map[string]*discordgo.Role)
	for _, role := range g.Roles {
		roleMap[role.ID] = role
		if role.ID == g.ID {
			everyoneRole = role
		}
	}

	var perms int64

	// Apply everyone role
	if everyoneRole != nil {
		perms |= everyoneRole.Permissions
	}

	// Apply member roles
	for _, role := range m.Roles {
		if r, ok := roleMap[role]; ok {
			perms |= r.Permissions
		}
	}

	return perms
}

// Returns the permissions of a member in a channel
func MemberChannelPerms(basePerms int64, g *discordgo.Guild, m *discordgo.Member, c *discordgo.Channel) int64 {
	var perms = basePerms // Start with base perms

	if perms&discordgo.PermissionAdministrator == discordgo.PermissionAdministrator {
		return discordgo.PermissionAll // ADMINISTRATOR overrides all
	}

	// Now we have the base everyone perms, apply the rest of the perms in hierarchy order
	var everyoneOverwrite *discordgo.PermissionOverwrite
	var roleOverwrites []*discordgo.PermissionOverwrite
	var memberOverwrites *discordgo.PermissionOverwrite

	for _, overwrite := range c.PermissionOverwrites {
		if overwrite.Type == discordgo.PermissionOverwriteTypeRole && overwrite.ID == g.ID {
			everyoneOverwrite = overwrite
		} else if overwrite.Type == discordgo.PermissionOverwriteTypeRole {
			if slices.Contains(m.Roles, overwrite.ID) {
				roleOverwrites = append(roleOverwrites, overwrite)
			}
		} else if overwrite.Type == discordgo.PermissionOverwriteTypeMember && overwrite.ID == m.User.ID {
			if overwrite.ID == m.User.ID {
				memberOverwrites = overwrite
			}
		}
	}

	// First, apply everyone overwrite
	if everyoneOverwrite != nil {
		perms &= ^everyoneOverwrite.Deny
		perms |= everyoneOverwrite.Allow
	}

	// Next, apply role overwrites
	for _, overwrite := range roleOverwrites {
		perms &= ^overwrite.Deny
		perms |= overwrite.Allow
	}

	// Finally, apply member overwrite
	if memberOverwrites != nil {
		perms &= ^memberOverwrites.Deny
		perms |= memberOverwrites.Allow
	}

	return perms
}

// Check if a permission int64 has a permission
func CheckPermission(basePerms int64, permission int64) bool {
	if basePerms&permission != discordgo.PermissionManageMessages && basePerms&discordgo.PermissionAdministrator != discordgo.PermissionAdministrator {
		return false
	}
	return true
}

// Check if a permission list has all permissions
func CheckAllPermissions(basePerms int64, permissions []int64) bool {
	// Check if admin, if so, everything is allowed
	if basePerms&discordgo.PermissionAdministrator == discordgo.PermissionAdministrator {
		return true
	}

	// Check if all permissions are allowed
	for _, perm := range permissions {
		if basePerms&perm != perm {
			return false
		}
	}

	return true
}
