package utils

import (
	"fmt"
	"strings"

	"github.com/google/uuid"
)

func IsValidID(value string, prefix string) bool {
	// Trim any whitespace that might have been added
	value = strings.TrimSpace(value)

	// Check if the value starts with the expected prefix
	expectedPrefix := prefix + "-"
	if !strings.HasPrefix(value, expectedPrefix) {
		return false
	}

	// Extract the UUID part after the prefix
	idUUID := strings.TrimPrefix(value, expectedPrefix)

	fmt.Println("idUUID ->", idUUID)
	fmt.Println("idUUID length ->", len(idUUID))
	fmt.Println("prefix ->", prefix)

	_, err := uuid.Parse(idUUID)
	if err != nil {
		fmt.Println("err ->", err)
	}
	return err == nil
}

func GenerateID(prefix string) string {
	return fmt.Sprintf("%s-%s", prefix, uuid.New().String())
}
