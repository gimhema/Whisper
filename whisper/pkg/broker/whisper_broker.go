package broker

import (
	"fmt"
	"whisper/pkg/common"
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
}