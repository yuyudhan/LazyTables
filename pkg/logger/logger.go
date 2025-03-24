// FilePath: pkg/logger/logger.go

// Package logger provides logging functionality for the LazyTables application.
// It supports different log levels, log file output, and console output for debugging.

package logger

import (
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

// Log levels
const (
	LevelDebug = iota
	LevelInfo
	LevelWarn
	LevelError
)

var (
	// Default level is Info
	logLevel = LevelInfo

	// Logger instances
	debugLogger *log.Logger
	infoLogger  *log.Logger
	warnLogger  *log.Logger
	errorLogger *log.Logger

	// Output file
	logFile *os.File

	// Log file path
	LogFilePath string
)

// Init initializes the logger with the specified log level and output location.
// If debugMode is true, logs will also be written to stdout.
func Init(debugMode bool) (string, error) {
	if debugMode {
		logLevel = LevelDebug
	}

	// Create logs directory in user's home directory
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("failed to get home directory: %w", err)
	}

	logDir := filepath.Join(homeDir, ".lazytables", "logs")
	if err := os.MkdirAll(logDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create log directory: %w", err)
	}

	// Create log file with timestamp
	timestamp := time.Now().Format("2006-01-02-15-04-05")
	LogFilePath = filepath.Join(logDir, fmt.Sprintf("lazytables-%s.log", timestamp))

	logFile, err = os.Create(LogFilePath)
	if err != nil {
		return "", fmt.Errorf("failed to create log file: %w", err)
	}

	// Set writers based on debug mode
	var debugOutput, infoOutput, warnOutput, errorOutput io.Writer

	if debugMode {
		debugOutput = io.MultiWriter(os.Stdout, logFile)
		infoOutput = io.MultiWriter(os.Stdout, logFile)
		warnOutput = io.MultiWriter(os.Stdout, logFile)
		errorOutput = io.MultiWriter(os.Stderr, logFile)
	} else {
		debugOutput = logFile
		infoOutput = logFile
		warnOutput = logFile
		errorOutput = io.MultiWriter(os.Stderr, logFile)
	}

	// Initialize loggers
	debugLogger = log.New(debugOutput, "DEBUG: ", log.Ldate|log.Ltime)
	infoLogger = log.New(infoOutput, "INFO: ", log.Ldate|log.Ltime)
	warnLogger = log.New(warnOutput, "WARN: ", log.Ldate|log.Ltime)
	errorLogger = log.New(errorOutput, "ERROR: ", log.Ldate|log.Ltime)

	return LogFilePath, nil
}

// Close closes the log file.
func Close() error {
	if logFile != nil {
		return logFile.Close()
	}
	return nil
}

// addFileInfo adds file and line information to log messages
func addFileInfo() string {
	_, file, line, ok := runtime.Caller(3) // Skip 3 frames to get to the caller
	if !ok {
		return ""
	}
	// Extract just the filename, not the full path
	parts := strings.Split(file, "/")
	file = parts[len(parts)-1]
	return fmt.Sprintf("[%s:%d] ", file, line)
}

// Debug logs a debug message if debug logging is enabled.
func Debug(format string, v ...interface{}) {
	if logLevel <= LevelDebug {
		debugLogger.Printf(addFileInfo()+format, v...)
	}
}

// Info logs an info message.
func Info(format string, v ...interface{}) {
	if logLevel <= LevelInfo {
		infoLogger.Printf(addFileInfo()+format, v...)
	}
}

// Warn logs a warning message.
func Warn(format string, v ...interface{}) {
	if logLevel <= LevelWarn {
		warnLogger.Printf(addFileInfo()+format, v...)
	}
}

// Error logs an error message.
func Error(format string, v ...interface{}) {
	if logLevel <= LevelError {
		errorLogger.Printf(addFileInfo()+format, v...)
	}
}
