package tasks

import (
	"encoding/json"
	"fmt"
	"splashtail/state"
	"sync"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

type MutLogger struct {
	sync.Mutex
	taskId string
}

func (m *MutLogger) add(p []byte) error {
	if state.Config.Meta.DebugTaskLogger {
		state.Logger.Debug("add called", zap.String("taskId", m.taskId))
	}
	defer m.Unlock()
	m.Lock()

	var data map[string]any

	err := json.Unmarshal(p, &data)

	if err != nil {
		return fmt.Errorf("failed to unmarshal json: %w", err)
	}

	// For us, this is just an array append of the json
	_, err = state.Pool.Exec(state.Context, "UPDATE tasks SET statuses = array_append(statuses, $1), last_updated = NOW() WHERE task_id = $2", data, m.taskId)

	if err != nil {
		return fmt.Errorf("failed to update statuses: %w", err)
	}

	return nil
}

func (m *MutLogger) Write(p []byte) (n int, err error) {
	if state.Config.Meta.DebugTaskLogger {
		state.Logger.Debug("Write called", zap.String("taskId", m.taskId))
	}

	err = m.add(p)

	if err != nil {
		state.Logger.Error("[dwWriter] Failed to add to buffer", zap.Error(err), zap.String("taskId", m.taskId))
	}

	return len(p), err
}

func (m *MutLogger) Sync() error {
	return nil
}

func NewTaskLogger(taskId string) (*zap.Logger, *MutLogger) {
	ml := &MutLogger{
		taskId: taskId,
	}

	logger := zap.New(zapcore.NewCore(
		zapcore.NewJSONEncoder(zap.NewProductionEncoderConfig()),
		ml,
		zapcore.DebugLevel,
	))
	return logger, ml
}
