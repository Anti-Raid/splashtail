package silverpelt

import (
	"context"
	"errors"
	"fmt"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

// Returns all configurations in a guild
func GetAllCommandConfigurationsForGuild(
	ctx context.Context,
	pool *pgxpool.Pool,
	guildID string,
) ([]GuildCommandConfiguration, error) {
	rows, err := pool.Query(
		ctx,
		"SELECT id, guild_id, command, perms, disabled FROM guild_command_configurations WHERE guild_id = $1",
		guildID,
	)

	if errors.Is(err, pgx.ErrNoRows) {
		return []GuildCommandConfiguration{}, nil
	}

	if err != nil {
		return nil, fmt.Errorf("failed to query guild_command_configuration: %w", err)
	}

	recs, err := pgx.CollectRows(rows, pgx.RowToStructByName[GuildCommandConfiguration])

	if errors.Is(err, pgx.ErrNoRows) {
		return []GuildCommandConfiguration{}, nil
	}

	if err != nil {
		return nil, fmt.Errorf("failed to collect rows: %w", err)
	}

	return recs, nil
}

// Returns all configurations of a command
func GetAllCommandConfigurations(
	ctx context.Context,
	pool *pgxpool.Pool,
	guildID string,
	name string,
) ([]*GuildCommandConfiguration, error) {
	permutations := PermuteCommandNames(name)

	configs := make([]*GuildCommandConfiguration, 0, len(permutations))

	for _, permutation := range permutations {
		var rec GuildCommandConfiguration
		err := pool.QueryRow(
			ctx,
			"SELECT id, guild_id, command, perms, disabled FROM guild_command_configurations WHERE guild_id = $1 AND command = $2",
			guildID, permutation,
		).Scan(
			&rec.ID,
			&rec.GuildID,
			&rec.Command,
			&rec.Perms,
			&rec.Disabled,
		)

		if errors.Is(err, pgx.ErrNoRows) {
			continue
		}

		if err != nil {
			return nil, fmt.Errorf("failed to query guild_command_configuration: %w", err)
		}

		configs = append(configs, &rec)
	}

	return configs, nil
}
