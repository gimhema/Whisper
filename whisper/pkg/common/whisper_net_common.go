package common

import (
	"fmt"
	"net"
	"syscall"
)

type TCPServer struct {
	Address string
	listener net.Listener
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

	// syscall-level으로 non-blocking 설정
	rawConn, err := conn.(*net.TCPConn).File()
	if err != nil {
		fmt.Println("Failed to get raw connection:", err)
		return
	}
	defer rawConn.Close()

	// Set the file descriptor to non-blocking
	if err := syscall.SetNonblock(int(rawConn.Fd()), true); err != nil {
		fmt.Println("Failed to set non-blocking mode:", err)
		return
	}

	buf := make([]byte, 1024)
	for {
		n, err := conn.Read(buf)
		if err != nil {
			// non-blocking이므로 일시적 에러는 무시
			if isTemporary(err) {
				continue
			}
			fmt.Println("Read error:", err)
			return
		}
		if n > 0 {
			data := buf[:n]
			fmt.Printf("Received: %s\n", string(data))
			conn.Write(data) // 에코
		}
	}
}

func isTemporary(err error) bool {
	if nerr, ok := err.(net.Error); ok && nerr.Temporary() {
		return true
	}
	return false
}

// func main() {
// 	server := NewTCPServer(":9000")
// 	if err := server.Run(); err != nil {
// 		fmt.Println("Server error:", err)
// 		os.Exit(1)
// 	}
// }
