package common

import (
	"fmt"
	"net"
	"syscall"
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
	defer conn.Close()

	rawConn, err := conn.(*net.TCPConn).File()
	if err != nil {
		fmt.Println("Failed to get raw connection:", err)
		return
	}
	defer rawConn.Close()

	syscall.SetNonblock(int(rawConn.Fd()), true)

	buf := make([]byte, 1024)
	for {
		n, err := conn.Read(buf)
		if err != nil {
			if isTemporary(err) {
				continue
			}
			fmt.Println("Read error:", err)
			return
		}
		if n > 0 {
			data := buf[:n]
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


