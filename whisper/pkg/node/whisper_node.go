package node

import (
	"fmt"
	"whisper/pkg/common"
)

type Node struct {
	tcpConn common.TCPServer
}

func CreateNode() *Node {
	_tcpConn := common.NewTCPServer(":8080")
	return &Node {
		tcpConn: *_tcpConn,
	}
}

func (broker *Node) Run() {
	fmt.Println("Run Message Node")
}