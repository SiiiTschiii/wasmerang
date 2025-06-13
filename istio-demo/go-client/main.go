package main

import (
	"fmt"
	"io"
	"net"
	"net/http"
	"strings"
	"time"
)

func main() {
	fmt.Println("Starting Go client - all traffic will be intercepted by WASM filter")
	fmt.Println("Expected behavior: Traffic routed to egress1 (even IP) or egress2 (odd IP)")
	
	// Test various types of connections that should all be intercepted
	go testTCPConnections()
	go testHTTPConnections()
	
	// Keep the main goroutine alive
	select {}
}

func testTCPConnections() {
	// Test raw TCP connections - these should ALL be intercepted and rerouted
	destinations := []string{
		"httpbin.org:80",           // External HTTP
		"www.example.com:80",       // External HTTP 
		"www.google.com:443",       // External HTTPS
	}
	
	for {
		for _, dest := range destinations {
			fmt.Printf("\n=== Testing TCP connection to %s ===\n", dest)
			fmt.Printf("WASM filter should intercept this and route to egress1 or egress2\n")
			
			conn, err := net.DialTimeout("tcp", dest, 10*time.Second)
			if err != nil {
				fmt.Printf("❌ TCP connection failed to %s: %v\n", dest, err)
				continue
			}
			
			fmt.Printf("✅ TCP connection established to %s\n", dest)
			
			// Send HTTP request to see which egress service handles it
			if strings.Contains(dest, ":80") || strings.Contains(dest, ":8080") {
				host := strings.Split(dest, ":")[0]
				request := fmt.Sprintf("GET / HTTP/1.1\r\nHost: %s\r\nConnection: close\r\n\r\n", host)
				conn.Write([]byte(request))
				
				// Read response to identify which egress service handled the request
				buffer := make([]byte, 2048)
				conn.SetReadDeadline(time.Now().Add(5 * time.Second))
				n, err := conn.Read(buffer)
				if err != nil {
					fmt.Printf("⚠️  Read timeout/error: %v\n", err)
				} else {
					response := string(buffer[:n])
					if strings.Contains(response, "egress1") {
						fmt.Printf("🎯 SUCCESS! Traffic intercepted and routed to EGRESS1\n")
					} else if strings.Contains(response, "egress2") {
						fmt.Printf("🎯 SUCCESS! Traffic intercepted and routed to EGRESS2\n")
					} else {
						fmt.Printf("📄 Response received (first 200 chars): %s\n", response[:min(200, len(response))])
					}
				}
			} else {
				// For HTTPS, just log the successful connection
				fmt.Printf("🔒 HTTPS connection established (interception should be visible in logs)\n")
			}
			
			conn.Close()
			time.Sleep(3 * time.Second)
		}
		fmt.Printf("\n--- Waiting 20 seconds before next round ---\n")
		time.Sleep(20 * time.Second)
	}
}

func testHTTPConnections() {
	// Test HTTP connections to external sites  
	// These might go through HTTP connection manager but we'll try them anyway
	urls := []string{
		"http://httpbin.org/ip",          // External HTTP
		"http://www.example.com/",        // External HTTP
	}
	
	for {
		for _, url := range urls {
			fmt.Printf("\n=== Testing HTTP request to %s ===\n", url)
			fmt.Printf("Note: HTTP requests may bypass TCP proxy filter chains\n")
			
			resp, err := http.Get(url)
			if err != nil {
				fmt.Printf("HTTP request error to %s: %v\n", url, err)
				continue
			}
			body, _ := io.ReadAll(resp.Body)
			resp.Body.Close()
			
			response := string(body)
			if strings.Contains(response, "egress1") {
				fmt.Printf("✅ HTTP INTERCEPTED! Traffic routed to EGRESS1\n")
			} else if strings.Contains(response, "egress2") {
				fmt.Printf("✅ HTTP INTERCEPTED! Traffic routed to EGRESS2\n")
			} else {
				fmt.Printf("Response from %s: %s\n", url, response[:min(150, len(response))])
			}
			time.Sleep(3 * time.Second)
		}
		time.Sleep(15 * time.Second)
	}
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
