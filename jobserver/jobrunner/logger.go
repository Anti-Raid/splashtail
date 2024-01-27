package jobrunner

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"

	"github.com/jackc/pgx/v5/pgxpool"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

type MutLogger struct {
	sync.Mutex
	taskId string
	pool   *pgxpool.Pool
	ctx    context.Context
	logger *zap.Logger
}

func (m *MutLogger) add(p []byte) error {
	defer m.Unlock()
	m.Lock()

	var data map[string]any

	err := json.Unmarshal(p, &data)

	if err != nil {
		return fmt.Errorf("failed to unmarshal json: %w", err)
	}

	// For us, this is just an array append of the json
	_, err = m.pool.Exec(m.ctx, "UPDATE tasks SET statuses = array_append(statuses, $1), last_updated = NOW() WHERE task_id = $2", data, m.taskId)

	if err != nil {
		return fmt.Errorf("failed to update statuses: %w", err)
	}

	return nil
}

func (m *MutLogger) Write(p []byte) (n int, err error) {
	err = m.add(p)

	if err != nil {
		m.logger.Error("[dwWriter] Failed to add to buffer", zap.Error(err), zap.String("taskId", m.taskId))
	}

	return len(p), err
}

func (m *MutLogger) Sync() error {
	return nil
}

func NewTaskLogger(taskId string, pool *pgxpool.Pool, ctx context.Context, baseLogger *zap.Logger) (*zap.Logger, *MutLogger) {
	ml := &MutLogger{
		taskId: taskId,
		pool:   pool,
		ctx:    ctx,
		logger: baseLogger,
	}

	logger := zap.New(zapcore.NewCore(
		zapcore.NewJSONEncoder(zap.NewProductionEncoderConfig()),
		ml,
		zapcore.DebugLevel,
	))
	return logger, ml
}
