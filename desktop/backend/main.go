package main

import (
	"log"
	"time"
)

func main() {
	currentTime := time.Now()
	log.Printf("Current time: %s", currentTime.Format(time.RFC1123))
}
