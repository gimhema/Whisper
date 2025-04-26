package broker

import (
	"fmt"
	"whisper/pkg/common"
	"os"
)

type Broker struct {
	tcpConn common.TCPServer
}

func CreateBroker() *Broker {
	_tcpConn := common.NewTCPServer(":8080")
	return &Broker{
		tcpConn: *_tcpConn,
	}
}

func (broker *Broker) Run() {
	fmt.Println("Run Message Broker")

	if err := broker.tcpConn.Run(); err != nil {
		fmt.Println("Server error:", err)
		os.Exit(1)
	}
}