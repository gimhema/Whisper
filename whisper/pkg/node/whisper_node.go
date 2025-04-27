package node

import (
	"fmt"
	"net"
)

type Node struct {
	conn net.Conn
	id   string
	subscribers map[string]func(string)
}


func ConnectToBroker(address string) (*Node, error) {
	conn, err := net.Dial("tcp", address)
	if err != nil {
		return nil, err
	}
	node := &Node{
		conn: conn,
		id:   conn.LocalAddr().String(),
	}
	return node, nil
}

func (n *Node) Publish(topic string, message string) error {
	payload := fmt.Sprintf("PUB %s %s\n", topic, message)
	_, err := n.conn.Write([]byte(payload))
	return err
}

func (n *Node) Subscribe(topic string, handler func(string)) error {
	if n.subscribers == nil {
		n.subscribers = make(map[string]func(string))
	}
	n.subscribers[topic] = handler

	payload := fmt.Sprintf("SUB %s\n", topic)
	_, err := n.conn.Write([]byte(payload))
	return err
}

func (n *Node) Listen() {
	buf := make([]byte, 1024)
	for {
		nRead, err := n.conn.Read(buf)
		if err != nil {
			fmt.Println("Read error:", err)
			break
		}
		if nRead > 0 {
			msg := string(buf[:nRead])
			fmt.Println("Received from broker:", msg)

			var command, topic, body string
			fmt.Sscanf(msg, "%s %s %[^\n]", &command, &topic, &body)
			if command == "MSG" {
				if handler, ok := n.subscribers[topic]; ok {
					handler(body)
				} else {
					fmt.Println("No handler for topic:", topic)
				}
			}
		}
	}
}

/*
func main() {
	node, err := ConnectToBroker("localhost:8080")
	if err != nil {
		panic(err)
	}

	go node.Listen() // 메시지 수신 비동기 처리

	node.Subscribe("news")
	time.Sleep(time.Second)
	node.Publish("news", "hello from node")

	select {} // main 종료 방지
}

*/