package node

import (
	"fmt"
)

type Node struct {
	
}

func CreateNode() *Node {
	return &Node {

	}
}

func (broker *Node) Run() {
	fmt.Println("Run Message Node")
}