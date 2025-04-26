package node

import (
	"fmt"
	"whisper/pkg/common"
	"os"
	"net"
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
	fmt.Println("Run Message Broker")

	node.tcpConn.SetMessageHandler(node.handleMessage)

	if err := node.tcpConn.Run(); err != nil {
		fmt.Println("Server error:", err)
		os.Exit(1)
	}
}

func (node *Node) handleMessage(conn net.Conn, data []byte) {
	// 예: 메시지를 처리하거나 라우팅
	msg := string(data)
	fmt.Printf("Broker received: %s\n", msg)

	// 예: 클라이언트에 응답 보내기
	conn.Write([]byte("ACK: " + msg))
}