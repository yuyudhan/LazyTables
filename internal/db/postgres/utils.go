// FilePath: internal/db/postgres/utils.go

package postgres

import (
	"strings"
)

// Helper function to determine if byte slice likely contains binary data
func isBinary(data []byte) bool {
	// A simple heuristic: if data contains many non-printable characters,
	// it's likely binary
	nonPrintable := 0
	sampleSize := len(data)
	if sampleSize > 100 {
		sampleSize = 100 // Check at most 100 bytes
	}

	for i := 0; i < sampleSize; i++ {
		if data[i] < 32 && !isPrintableControl(data[i]) {
			nonPrintable++
		}
	}

	// Consider binary if >15% non-printable chars
	return nonPrintable > sampleSize/6
}

// isPrintableControl returns true for whitespace control chars
func isPrintableControl(b byte) bool {
	return b == '\n' || b == '\r' || b == '\t'
}

// Helper function to get the type of SQL query
func getQueryType(query string) string {
	query = strings.TrimSpace(query)
	upperQuery := strings.ToUpper(query)

	if strings.HasPrefix(upperQuery, "SELECT") {
		return "SELECT"
	} else if strings.HasPrefix(upperQuery, "INSERT") {
		return "INSERT"
	} else if strings.HasPrefix(upperQuery, "UPDATE") {
		return "UPDATE"
	} else if strings.HasPrefix(upperQuery, "DELETE") {
		return "DELETE"
	} else if strings.HasPrefix(upperQuery, "CREATE") {
		return "CREATE"
	} else if strings.HasPrefix(upperQuery, "ALTER") {
		return "ALTER"
	} else if strings.HasPrefix(upperQuery, "DROP") {
		return "DROP"
	} else if strings.HasPrefix(upperQuery, "SHOW") {
		return "SHOW"
	} else if strings.HasPrefix(upperQuery, "EXPLAIN") {
		return "EXPLAIN"
	}

	return "UNKNOWN"
}

// Helper function to return the first non-empty string
func firstNonEmpty(values ...string) string {
	for _, v := range values {
		if v != "" {
			return v
		}
	}
	return ""
}
