package node

import (
	"fmt"
	"whisper/pkg/common"
	"os"
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

func (node *Node) Run() {
	fmt.Println("Run Message Node")

	if err := node.tcpConn.Run(); err != nil {
		fmt.Println("Server error:", err)
		os.Exit(1)
	}
}