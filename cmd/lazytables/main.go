// FilePath: cmd/lazytables/main.go

// This is the entry point for the LazyTables application.
// It initializes the application, handles command line arguments,
// and starts the terminal UI.

package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/yuyudhan/LazyTables/internal/app"
	"github.com/yuyudhan/LazyTables/pkg/logger"
)

const (
	appName    = "LazyTables"
	appVersion = "0.1.0"
)

func main() {
	// Parse command line flags
	versionFlag := flag.Bool("version", false, "Display version information")
	debugFlag := flag.Bool("debug", false, "Enable debug mode")
	configPath := flag.String("config", "", "Path to config file")
	flag.Parse()

	// Handle version flag
	if *versionFlag {
		fmt.Printf("%s v%s\n", appName, appVersion)
		os.Exit(0)
	}

	// Setup logging
	logPath, err := logger.Init(*debugFlag)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error setting up logging: %v\n", err)
		os.Exit(1)
	}
	defer logger.Close()

	logger.Info("Starting %s v%s", appName, appVersion)

	// Initialize and run the application
	app, err := app.New(*configPath, *debugFlag)
	if err != nil {
		logger.Error("Failed to initialize application: %v", err)
		fmt.Fprintf(os.Stderr, "Error initializing application: %v\n", err)
		fmt.Fprintf(os.Stderr, "For more details, check the log at: %s\n", logPath)
		os.Exit(1)
	}

	logger.Info("Application initialized successfully")

	if err := app.Run(); err != nil {
		logger.Error("Application error: %v", err)
		fmt.Fprintf(os.Stderr, "Error running application: %v\n", err)
		fmt.Fprintf(os.Stderr, "For more details, check the log at: %s\n", logPath)
		os.Exit(1)
	}
}
