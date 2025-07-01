package main

import (
	"io"
	"log"
	"net"
	"os"
	"strings"
	"time"
)

const (
	// Hardcoded destination for now - httpbin.org
	DESTINATION_HOST = "44.207.188.95"
	
	// Two listeners for different protocols
	HTTPS_LISTEN_PORT = "8080"  // Listen for HTTPS traffic, forward to port 443
	HTTP_LISTEN_PORT  = "8081"  // Listen for HTTP traffic, forward to port 80
	
	HTTPS_DEST_PORT = "443"     // Forward HTTPS traffic to port 443
	HTTP_DEST_PORT  = "80"      // Forward HTTP traffic to port 80
)

// isConnectionClosed checks if an error is due to a closed network connection
func isConnectionClosed(err error) bool {
	return strings.Contains(err.Error(), "use of closed network connection") ||
		   strings.Contains(err.Error(), "broken pipe") ||
		   strings.Contains(err.Error(), "connection reset by peer")
}

func main() {
	name := os.Getenv("SERVER_NAME")
	if name == "" {
		name = "unknown-egress-router"
	}
	
	log.Printf("[%s] 🚀 Starting dual-protocol TCP egress router", name)
	log.Printf("[%s] 🔒 HTTPS listener: port %s -> %s:%s", name, HTTPS_LISTEN_PORT, DESTINATION_HOST, HTTPS_DEST_PORT)
	log.Printf("[%s] 🌐 HTTP listener: port %s -> %s:%s", name, HTTP_LISTEN_PORT, DESTINATION_HOST, HTTP_DEST_PORT)
	
	// Start HTTPS listener (port 8080 -> 443)
	go startListener(name, HTTPS_LISTEN_PORT, HTTPS_DEST_PORT, "HTTPS")
	
	// Start HTTP listener (port 8081 -> 80)
	go startListener(name, HTTP_LISTEN_PORT, HTTP_DEST_PORT, "HTTP")
	
	// Keep main goroutine alive
	select {}
}

func startListener(serverName, listenPort, destPort, protocol string) {
	listener, err := net.Listen("tcp", ":"+listenPort)
	if err != nil {
		log.Fatalf("[%s] ❌ Failed to start %s TCP server on port %s: %v", serverName, protocol, listenPort, err)
	}
	defer listener.Close()
	
	log.Printf("[%s] ✅ %s TCP listener ready on port %s, forwarding to %s:%s", serverName, protocol, listenPort, DESTINATION_HOST, destPort)
	
	for {
		conn, err := listener.Accept()
		if err != nil {
			log.Printf("[%s] ❌ Error accepting %s connection on port %s: %v", serverName, protocol, listenPort, err)
			continue
		}
		
		// Handle each connection in a goroutine
		go handleConnection(conn, serverName, destPort, protocol)
	}
}

func handleConnection(conn net.Conn, serverName, destPort, protocol string) {
	defer conn.Close()
	
	start := time.Now()
	clientAddr := conn.RemoteAddr().String()
	
	log.Printf("[%s] 🎯 NEW %s CONNECTION from %s", serverName, protocol, clientAddr)
	log.Printf("[%s] 📡 Local address: %s", serverName, conn.LocalAddr().String())
	log.Printf("[%s] 🔀 Bridging to %s:%s", serverName, DESTINATION_HOST, destPort)
	
	// Connect to the destination server
	destConn, err := net.DialTimeout("tcp", DESTINATION_HOST+":"+destPort, 10*time.Second)
	if err != nil {
		log.Printf("[%s] ❌ Failed to connect to destination %s:%s: %v", serverName, DESTINATION_HOST, destPort, err)
		return
	}
	defer destConn.Close()
	
	log.Printf("[%s] ✅ Connected to destination %s:%s", serverName, DESTINATION_HOST, destPort)
	
	// Start bidirectional copying
	done := make(chan struct{}, 2)
	
	// Copy from client to destination
	go func() {
		defer func() { 
			destConn.Close() // Close destination to signal the other goroutine
			done <- struct{}{}
		}()
		bytesWritten, err := io.Copy(destConn, conn)
		if err != nil {
			// Only log if it's not a "use of closed network connection" error
			if !isConnectionClosed(err) {
				log.Printf("[%s] ⚠️  Error copying client->dest: %v", serverName, err)
			}
		} else {
			log.Printf("[%s] 📤 Copied %d bytes client->dest", serverName, bytesWritten)
		}
	}()
	
	// Copy from destination to client
	go func() {
		defer func() { 
			conn.Close() // Close client connection to signal the other goroutine
			done <- struct{}{}
		}()
		bytesWritten, err := io.Copy(conn, destConn)
		if err != nil {
			// Only log if it's not a "use of closed network connection" error
			if !isConnectionClosed(err) {
				log.Printf("[%s] ⚠️  Error copying dest->client: %v", serverName, err)
			}
		} else {
			log.Printf("[%s] 📥 Copied %d bytes dest->client", serverName, bytesWritten)
		}
	}()
	
	// Wait for both directions to complete
	<-done
	<-done
	
	duration := time.Since(start)
	log.Printf("[%s] ✅ Connection bridged for %v", serverName, duration)
	log.Printf("[%s] ==========================================", serverName)
}

