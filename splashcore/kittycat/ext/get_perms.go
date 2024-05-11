package ext

import (
	"context"
	"fmt"

	perms "github.com/infinitybotlist/kittycat/go"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

type sp struct {
	ID    string   `db:"role_id"`
	Index int32    `db:"index"`
	Perms []string `db:"perms"`
}

func GetUserPermi(ctx context.Context, pool *pgxpool.Pool, guildId, userId string) (*perms.StaffPermissions, error) {
	var roles []string
	var permOverrides []string

	err := pool.QueryRow(ctx, "SELECT roles, perm_overrides FROM guild_members WHERE guild_id = $1 AND user_id = $2", guildId, userId).Scan(&roles, &permOverrides)

	if err != nil {
		return nil, fmt.Errorf("failed to get guild member: %w", err)
	}

	rows, err := pool.Query(ctx, "SELECT role_id, index, perms FROM guild_roles WHERE id = ANY($1)", roles)

	if err != nil {
		return nil, fmt.Errorf("failed to get staff positions: %w", err)
	}

	defer rows.Close()

	posFull, err := pgx.CollectRows(rows, pgx.RowToAddrOfStructByName[sp])

	if err != nil {
		return nil, fmt.Errorf("failed to collect rows: %w", err)
	}

	var sp = perms.StaffPermissions{
		PermOverrides: perms.PFSS(permOverrides),
		UserPositions: make([]perms.PartialStaffPosition, len(posFull)),
	}
	for _, pos := range posFull {
		sp.UserPositions = append(sp.UserPositions, perms.PartialStaffPosition{
			ID:    pos.ID,
			Perms: perms.PFSS(pos.Perms),
			Index: pos.Index,
		})
	}

	return &sp, nil
}
