package main

import (
	"fmt"
	"os"

	"whisper/pkg/broker"
	"whisper/pkg/common"
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
			connInfo := args[2]
			mode = common.BROKER
			fmt.Println("Broker mode")

			broker := broker.CreateBroker(connInfo)
			broker.Run()

		} else if val == "node" {
			connInfo := args[2]

			mode = common.NODE
			fmt.Println("Node mode")

			node, err := node.ConnectToBroker(connInfo)
			if err != nil {
				panic(err)
			}

			node.SetupHandler()

			go node.Listen()

			node.HandleUserInput()

			select {}

		} else {
			fmt.Println("Unsupported mode")
		}
	} else {
		fmt.Println("No arguments passed.")
	}

	fmt.Println("Selected mode:", mode)
}
