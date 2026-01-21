package utils

import (
	"encoding/json"
	"fmt"
)

func PrettyPrint(v interface{}) {
	json, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		fmt.Println(err)
	}
	fmt.Println(string(json))
}
