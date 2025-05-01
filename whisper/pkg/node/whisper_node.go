package node

import (
	"bufio"
	"fmt"
	"net"
	"os"
	"strings"
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

func handleChat (msg string) {
	fmt.Println("[Chat] Received:", msg)
}

func (n *Node) SetupHandler() {

	n.RegisterHandler("chat", handleChat)
}

// 메세지 송신
func (n *Node) Publish(topic string, message string) error {
	payload := fmt.Sprintf("PUB %s %s\n", topic, message)
	_, err := n.conn.Write([]byte(payload))
	return err
}

func (n *Node) HandleUserInput() {
	scanner := bufio.NewScanner(os.Stdin)
	fmt.Println("Type commands: (e.g., 'SUB topic', 'PUB topic message')")
	for scanner.Scan() {
		input := scanner.Text()
		parts := strings.SplitN(input, " ", 3) // 최대 3개로 분할
		if len(parts) < 2 {
			fmt.Println("Invalid command. Usage: SUB <topic> | PUB <topic> <message>")
			continue
		}
		command := strings.ToUpper(parts[0])
		topic := parts[1]

		switch command {
		case "SUB":
			err := n.Subscribe(topic)
			if err != nil {
				fmt.Println("Subscribe error:", err)
			} else {
				fmt.Println("Subscribed to topic:", topic)
			}
		case "PUB":
			if len(parts) < 3 {
				fmt.Println("Publish needs a topic and a message. Usage: PUB <topic> <message>")
				continue
			}
			message := parts[2]
			err := n.Publish(topic, message)
			if err != nil {
				fmt.Println("Publish error:", err)
			}
		default:
			fmt.Println("Unknown command:", command)
		}
	}
	if err := scanner.Err(); err != nil {
		fmt.Println("Input error:", err)
	}
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
			lines := strings.Split(strings.TrimSpace(msg), "\n")
			for _, line := range lines {
				parts := strings.SplitN(line, " ", 3)
				if len(parts) < 3 {
					fmt.Println("Malformed message:", line)
					continue
				}
				command, topic, body := parts[0], parts[1], parts[2]
			
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