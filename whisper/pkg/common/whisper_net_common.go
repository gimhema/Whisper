package common

import (
	"fmt"
	"net"
	"bufio"
	"strings"
)

type TCPServer struct {
	Address      string
	listener     net.Listener
	messageHandler func(conn net.Conn, data []byte)
}

func (s *TCPServer) SetMessageHandler(handler func(conn net.Conn, data []byte)) {
	s.messageHandler = handler
}


func NewTCPServer(address string) *TCPServer {
	return &TCPServer{
		Address: address,
	}
}

func (s *TCPServer) Run() error {
	// TCP 리스너 바인딩
	ln, err := net.Listen("tcp", s.Address)
	if err != nil {
		return fmt.Errorf("failed to listen: %w", err)
	}
	s.listener = ln
	fmt.Println("Server is listening on", s.Address)

	for {
		conn, err := s.listener.Accept()
		if err != nil {
			fmt.Println("Accept error:", err)
			continue
		}

		// 클라이언트 처리 고루틴 비동기 실행
		go s.handleConnection(conn)
	}
}

func (s *TCPServer) handleConnection(conn net.Conn) {
	reader := bufio.NewReader(conn)
	for {
		line, err := reader.ReadString('\n') // 한 줄 읽기
		if err != nil {
			if isTemporary(err) {
				continue
			}
			fmt.Println("Read error:", err)
			conn.Close()
			return
		}
		if len(line) > 0 {
			data := []byte(strings.TrimSpace(line)) // 줄바꿈 제거
			if s.messageHandler != nil {
				s.messageHandler(conn, data)
			}
		}
	}
}




func isTemporary(err error) bool {
	if nerr, ok := err.(net.Error); ok && nerr.Temporary() {
		return true
	}
	return false
}


