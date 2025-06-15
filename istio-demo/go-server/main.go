package main

import (
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"time"
)

func main() {
	name := os.Getenv("SERVER_NAME")
	if name == "" {
		name = "unknown-server"
	}
	
	// Start TCP server on port 80
	go func() {
		log.Printf("[%s] 🚀 Starting TCP server on port 80", name)
		listener, err := net.Listen("tcp", ":80")
		if err != nil {
			log.Fatalf("[%s] ❌ Failed to start TCP server on port 80: %v", name, err)
		}
		defer listener.Close()
		
		log.Printf("[%s] ✅ TCP server listening on port 80", name)
		
		for {
			conn, err := listener.Accept()
			if err != nil {
				log.Printf("[%s] ❌ Error accepting connection on port 80: %v", name, err)
				continue
			}
			
			// Handle each connection in a goroutine
			go handleConnection(conn, name, "80")
		}
	}()
	
	// Start TCP server on port 443
	log.Printf("[%s] 🚀 Starting TCP server on port 443", name)
	listener, err := net.Listen("tcp", ":443")
	if err != nil {
		log.Fatalf("[%s] ❌ Failed to start TCP server on port 443: %v", name, err)
	}
	defer listener.Close()
	
	log.Printf("[%s] ✅ TCP server listening on port 443", name)
	
	for {
		conn, err := listener.Accept()
		if err != nil {
			log.Printf("[%s] ❌ Error accepting connection on port 443: %v", name, err)
			continue
		}
		
		// Handle each connection in a goroutine
		go handleConnection(conn, name, "443")
	}
}

func handleConnection(conn net.Conn, serverName string, port string) {
	defer conn.Close()
	
	start := time.Now()
	clientAddr := conn.RemoteAddr().String()
	
	log.Printf("[%s] 🎯 NEW TCP CONNECTION on port %s from %s", serverName, port, clientAddr)
	log.Printf("[%s] 📡 Local address: %s", serverName, conn.LocalAddr().String())
	
	// Read data from the client
	buffer := make([]byte, 1024)
	n, err := conn.Read(buffer)
	if err != nil && err != io.EOF {
		log.Printf("[%s] ❌ Error reading from connection: %v", serverName, err)
		return
	}
	
	if n > 0 {
		clientData := string(buffer[:n])
		log.Printf("[%s] 📥 Received data on port %s: %q", serverName, port, clientData)
	}
	
	// Send response back to client
	response := fmt.Sprintf("Hello from %s on port %s! Time: %s\n", serverName, port, time.Now().Format(time.RFC3339))
	_, err = conn.Write([]byte(response))
	if err != nil {
		log.Printf("[%s] ❌ Error writing response: %v", serverName, err)
		return
	}
	
	duration := time.Since(start)
	log.Printf("[%s] ✅ Connection on port %s handled in %v", serverName, port, duration)
	log.Printf("[%s] ==========================================", serverName)
}

