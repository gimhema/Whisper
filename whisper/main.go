package main

import (
	"fmt"
	"os"

	"whisper/pkg/common"
	"whisper/pkg/broker"
	"whisper/pkg/node"
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

			broker := broker.CreateBroker()
			broker.Run()
			
		} else if val == "node" {
			mode = common.NODE
			fmt.Println("Node mode")
			
			node, err := node.ConnectToBroker("localhost:8080")
			if err != nil {
				panic(err)
			}

			go node.Listen()

			select {} 

		} else {
			fmt.Println("Unsupported mode")
		}
	} else {
		fmt.Println("No arguments passed.")
	}

	fmt.Println("Selected mode:", mode)
}
