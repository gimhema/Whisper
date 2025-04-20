package broker

import (
	"fmt"
)

type Broker struct {

}

func CreateBroker() *Broker {
	return &Broker {

	}
}

func (broker *Broker) Run() {
	fmt.Println("Run Message Broker")
}