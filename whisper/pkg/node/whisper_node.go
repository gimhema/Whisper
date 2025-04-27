package node

import (
	"fmt"
	"net"
)

type Node struct {
	conn        net.Conn
	id          string
	subscribers map[string]func(string) // 토픽별 메세지 핸들러
}

// 브로커에 연결
func ConnectToBroker(address string) (*Node, error) {
	conn, err := net.Dial("tcp", address)
	if err != nil {
		return nil, err
	}
	node := &Node{
		conn:        conn,
		id:          conn.LocalAddr().String(),
		subscribers: make(map[string]func(string)),
	}
	return node, nil
}

// 서버에 구독 요청
func (n *Node) Subscribe(topic string) error {
	payload := fmt.Sprintf("SUB %s\n", topic)
	_, err := n.conn.Write([]byte(payload))
	return err
}

// 토픽별 콜백 등록
func (n *Node) RegisterHandler(topic string, handler func(string)) {
	n.subscribers[topic] = handler
}

// func handleNews(msg string) {
// 	fmt.Println("[news] Received:", msg)
// }

func (n *Node) SetupHandler() {

	// node.RegisterHandler("news", handleNews)
	// node.RegisterHandler("sports", handleSports)
}

// 메세지 송신
func (n *Node) Publish(topic string, message string) error {
	payload := fmt.Sprintf("PUB %s %s\n", topic, message)
	_, err := n.conn.Write([]byte(payload))
	return err
}

// 메세지 수신 및 처리
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
					fmt.Println("No handler registered for topic:", topic)
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

	// 토픽별 콜백 함수 등록
	node.RegisterHandler("news", func(msg string) {
		fmt.Println("[news] Received:", msg)
	})

	node.RegisterHandler("sports", func(msg string) {
		fmt.Println("[sports] Received:", msg)
	})

	// 브로커에 구독 요청
	node.Subscribe("news")
	node.Subscribe("sports")

	go node.Listen()

	node.Publish("news", "Hello from node!")
	node.Publish("sports", "Soccer match at 8PM")

	select {} // 프로그램 종료 방지
}


*/