package main

import (
	"io"
	"log"
	"net"
	"os"
	"time"
)

const (
	// Hardcoded destination for now - httpbin.org
	DESTINATION_HOST = "54.198.84.155"
	DESTINATION_PORT = "443"
	LISTEN_PORT      = "8080"
)

func main() {
	name := os.Getenv("SERVER_NAME")
	if name == "" {
		name = "unknown-egress-router"
	}
	
	// Start TCP server on port 8080 (non-standard port)
	log.Printf("[%s] 🚀 Starting TCP egress router on port %s", name, LISTEN_PORT)
	listener, err := net.Listen("tcp", ":"+LISTEN_PORT)
	if err != nil {
		log.Fatalf("[%s] ❌ Failed to start TCP server on port %s: %v", name, LISTEN_PORT, err)
	}
	defer listener.Close()
	
	log.Printf("[%s] ✅ TCP egress router listening on port %s, forwarding to %s:%s", name, LISTEN_PORT, DESTINATION_HOST, DESTINATION_PORT)
	
	for {
		conn, err := listener.Accept()
		if err != nil {
			log.Printf("[%s] ❌ Error accepting connection on port %s: %v", name, LISTEN_PORT, err)
			continue
		}
		
		// Handle each connection in a goroutine
		go handleConnection(conn, name)
	}
}

func handleConnection(conn net.Conn, serverName string) {
	defer conn.Close()
	
	start := time.Now()
	clientAddr := conn.RemoteAddr().String()
	
	log.Printf("[%s] 🎯 NEW TCP CONNECTION from %s", serverName, clientAddr)
	log.Printf("[%s] 📡 Local address: %s", serverName, conn.LocalAddr().String())
	log.Printf("[%s] 🔀 Bridging to %s:%s", serverName, DESTINATION_HOST, DESTINATION_PORT)
	
	// Connect to the destination server
	destConn, err := net.DialTimeout("tcp", DESTINATION_HOST+":"+DESTINATION_PORT, 10*time.Second)
	if err != nil {
		log.Printf("[%s] ❌ Failed to connect to destination %s:%s: %v", serverName, DESTINATION_HOST, DESTINATION_PORT, err)
		return
	}
	defer destConn.Close()
	
	log.Printf("[%s] ✅ Connected to destination %s:%s", serverName, DESTINATION_HOST, DESTINATION_PORT)
	
	// Start bidirectional copying
	done := make(chan struct{}, 2)
	
	// Copy from client to destination
	go func() {
		defer func() { done <- struct{}{} }()
		bytesWritten, err := io.Copy(destConn, conn)
		if err != nil {
			log.Printf("[%s] ⚠️  Error copying client->dest: %v", serverName, err)
		} else {
			log.Printf("[%s] 📤 Copied %d bytes client->dest", serverName, bytesWritten)
		}
	}()
	
	// Copy from destination to client
	go func() {
		defer func() { done <- struct{}{} }()
		bytesWritten, err := io.Copy(conn, destConn)
		if err != nil {
			log.Printf("[%s] ⚠️  Error copying dest->client: %v", serverName, err)
		} else {
			log.Printf("[%s] 📥 Copied %d bytes dest->client", serverName, bytesWritten)
		}
	}()
	
	// Wait for either direction to complete
	<-done
	
	duration := time.Since(start)
	log.Printf("[%s] ✅ Connection bridged for %v", serverName, duration)
	log.Printf("[%s] ==========================================", serverName)
}

