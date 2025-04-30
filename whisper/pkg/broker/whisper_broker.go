package broker

import (
	"fmt"
	"net"
	"os"
	"strings"
	"whisper/pkg/common"
)

type Subscriber struct {
	conn net.Conn
	id   string
}

type Broker struct {
	tcpConn       common.TCPServer
	subscriptions map[string]map[string]*Subscriber
	subscribers   map[string]*Subscriber
}

func CreateBroker(connInfo string) *Broker {
	// _tcpConn := common.NewTCPServer(":8080")
	_tcpConn := common.NewTCPServer(connInfo)
	return &Broker{
		tcpConn:       *_tcpConn,
		subscriptions: make(map[string]map[string]*Subscriber),
		subscribers:   make(map[string]*Subscriber),
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
	msg := string(data)
	parts := strings.SplitN(msg, " ", 3)

	fmt.Println("Received Raw Message:", msg)

	if len(parts) < 2 {
		conn.Write([]byte("ERR invalid message format\n"))
		return
	}

	cmd := parts[0]
	topic := parts[1]

	switch cmd {
	case "SUB":
		broker.subscribe(conn, topic)
	case "PUB":
		if len(parts) < 3 {
			conn.Write([]byte("ERR missing message\n"))
			return
		}
		message := parts[2]
		broker.publish(topic, message)
	default:
		conn.Write([]byte("ERR unknown command\n"))
	}
}

func (broker *Broker) subscribe(conn net.Conn, topic string) {
	id := conn.RemoteAddr().String()
	sub := &Subscriber{conn: conn, id: id}

	// 등록
	if broker.subscriptions[topic] == nil {
		broker.subscriptions[topic] = make(map[string]*Subscriber)
	}
	broker.subscriptions[topic][id] = sub
	broker.subscribers[id] = sub

	conn.Write([]byte("SUBSCRIBED to " + topic + "\n"))
}

func (broker *Broker) publish(topic, message string) {

	fmt.Println("publish topic : ", topic)

	subs, ok := broker.subscriptions[topic]
	if !ok || len(subs) == 0 {
		fmt.Println("No subscribers for topic:", topic)
		return
	}
	for id, sub := range subs {
		_, err := sub.conn.Write([]byte("MSG " + topic + " " + message + "\n"))
		if err != nil {
			fmt.Println("Failed to send to subscriber:", id, "error:", err)
			delete(subs, id)
		}
	}
}
