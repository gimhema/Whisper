#!/bin/bash

# 이름을 인자로 받음
PROJECT_NAME=$1

if [ -z "$PROJECT_NAME" ]; then
  echo "Usage: $0 <project-name>"
  exit 1
fi

# 디렉토리 생성
mkdir -p "$PROJECT_NAME/pkg"
cd "$PROJECT_NAME" || exit

# go.mod 생성
go mod init "$PROJECT_NAME"

# main.go 생성
cat <<EOF > main.go
package main

import "fmt"

func main() {
    fmt.Println("Hello, $PROJECT_NAME!")
}
EOF

echo "Go project '$PROJECT_NAME' initialized!"