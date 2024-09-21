package lib

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"sync"
	"time"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

type LocalMutLogger struct {
	sync.Mutex
	taskId string
	logger *zap.Logger
}

var dbgtl = os.Getenv("DEBUG_TASK_LOGGER") == "true"

func (m *LocalMutLogger) add(p []byte) error {
	if dbgtl {
		m.logger.Debug("add called", zap.String("taskId", m.taskId))
	}
	defer m.Unlock()
	m.Lock()

	var data map[string]any

	err := json.Unmarshal(p, &data)

	if err != nil {
		return fmt.Errorf("failed to unmarshal json: %w", err)
	}

	// For us, this is just a print
	var output strings.Builder

	// Get the level
	level, ok := data["level"].(string)

	if !ok {
		level = fmt.Sprint(data["level"])
	}

	// Get the message
	message, ok := data["msg"].(string)

	if !ok {
		message = fmt.Sprint(data["msg"])
	}

	output.Write([]byte(fmt.Sprintf("%s: %s", level, message)))

	// Write fields
	for k, v := range data {
		if k == "level" || k == "msg" {
			continue
		}

		switch v := v.(type) {
		case float64:
			output.Write([]byte(fmt.Sprintf(", %s=%f", k, v)))
		case time.Duration:
			output.Write([]byte(fmt.Sprintf(", %s=%s", k, v.String())))
		default:
			output.Write([]byte(fmt.Sprintf(", %s=%s", k, v)))
		}
	}

	fmt.Println(output.String())

	return nil
}

func (m *LocalMutLogger) Write(p []byte) (n int, err error) {
	if dbgtl {
		m.logger.Debug("Write called", zap.String("taskId", m.taskId))
	}

	err = m.add(p)

	if err != nil {
		m.logger.Error("[dwWriter] Failed to add to buffer", zap.Error(err), zap.String("taskId", m.taskId))
	}

	return len(p), err
}

func (m *LocalMutLogger) Sync() error {
	return nil
}

func NewLocalLogger(taskId string, l *zap.Logger) (*zap.Logger, *LocalMutLogger) {
	ml := &LocalMutLogger{
		taskId: taskId,
		logger: l,
	}

	logger := zap.New(zapcore.NewCore(
		zapcore.NewJSONEncoder(zap.NewProductionEncoderConfig()),
		ml,
		zapcore.DebugLevel,
	))
	return logger, ml
}
