package broker

import (
	"fmt"
	"whisper/pkg/common"
	"os"
	"net"
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

	broker.tcpConn.SetMessageHandler(broker.handleMessage)

	if err := broker.tcpConn.Run(); err != nil {
		fmt.Println("Server error:", err)
		os.Exit(1)
	}
}

func (broker *Broker) handleMessage(conn net.Conn, data []byte) {
	// 예: 메시지를 처리하거나 라우팅
	msg := string(data)
	fmt.Printf("Broker received: %s\n", msg)

	// 예: 클라이언트에 응답 보내기
	conn.Write([]byte("ACK: " + msg))
}