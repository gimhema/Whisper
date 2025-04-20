package main

import (
	"fmt"
	"os"

	"whisper/pkg/common"
)

func main() {
	args := os.Args

	mode := common.DEFAULT
	var val string

	if len(args) > 1 {
		val = args[1]
		fmt.Println("Arguments:", val)

		if val == "broker" {
			mode = common.BROKER
			fmt.Println("Broker mode")
		} else if val == "node" {
			mode = common.NODE
			fmt.Println("Node mode")
		} else {
			fmt.Println("Unsupported mode")
		}
	} else {
		fmt.Println("No arguments passed.")
	}

	fmt.Println("Selected mode:", mode)
}
