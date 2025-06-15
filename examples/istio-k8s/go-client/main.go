package main

import (
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
)

func main() {
	fmt.Println("Starting Go client - testing WASM TCP filter with HTTP/HTTPS requests")
	fmt.Println("Expected behavior:")
	fmt.Println("  - HTTP requests to httpbin.org:80 → intercepted by WASM filter → routed to egress1/egress2:80")
	fmt.Println("  - HTTPS requests to httpbin.org:443 → intercepted by WASM filter → routed to egress1/egress2:443")
	fmt.Println("  - Even source IP last octet → egress1")
	fmt.Println("  - Odd source IP last octet → egress2")
	fmt.Println("")
	
	// Test HTTP and HTTPS requests
	go testHTTPConnection()
	go testHTTPSConnection()
	
	// Keep the main goroutine alive
	select {}
}

func testHTTPConnection() {
	// Test HTTP connection - should be intercepted by WASM filter and routed to egress TCP server
	url := "http://httpbin.org/ip"
	
	for {
		fmt.Printf("\n=== Testing HTTP request to %s ===\n", url)
		fmt.Printf("Expected: WASM filter intercepts and routes to egress1/egress2 port 80\n")
		
		resp, err := http.Get(url)
		if err != nil {
			fmt.Printf("❌ HTTP request error: %v\n", err)
			fmt.Printf("   This likely means WASM filter intercepted and routed to TCP server (connection failed)\n")
		} else {
			body, _ := io.ReadAll(resp.Body)
			resp.Body.Close()
			
			response := string(body)
			if strings.Contains(response, "egress1") {
				fmt.Printf("🎯 SUCCESS! HTTP traffic intercepted and routed to EGRESS1 port 80\n")
			} else if strings.Contains(response, "egress2") {
				fmt.Printf("🎯 SUCCESS! HTTP traffic intercepted and routed to EGRESS2 port 80\n")
			} else {
				fmt.Printf("⚠️  HTTP traffic may not be intercepted by WASM filter\n")
				fmt.Printf("Response: %s\n", response[:min(100, len(response))])
			}
		}
		
		fmt.Printf("Waiting 10 seconds before next HTTP request...\n")
		time.Sleep(10 * time.Second)
	}
}

func testHTTPSConnection() {
	// Test HTTPS connection - should be intercepted by WASM filter and routed to egress TCP server
	url := "https://httpbin.org/ip"
	
	for {
		fmt.Printf("\n=== Testing HTTPS request to %s ===\n", url)
		fmt.Printf("Expected: WASM filter intercepts and routes to egress1/egress2 port 443\n")
		
		resp, err := http.Get(url)
		if err != nil {
			fmt.Printf("❌ HTTPS request error: %v\n", err)
			fmt.Printf("   This likely means WASM filter intercepted and routed to TCP server (TLS handshake failed)\n")
		} else {
			body, _ := io.ReadAll(resp.Body)
			resp.Body.Close()
			
			response := string(body)
			if strings.Contains(response, "egress1") {
				fmt.Printf("🎯 SUCCESS! HTTPS traffic intercepted and routed to EGRESS1 port 443\n")
			} else if strings.Contains(response, "egress2") {
				fmt.Printf("🎯 SUCCESS! HTTPS traffic intercepted and routed to EGRESS2 port 443\n")
			} else {
				fmt.Printf("⚠️  HTTPS traffic may not be intercepted by WASM filter\n")
				fmt.Printf("Response: %s\n", response[:min(100, len(response))])
			}
		}
		
		fmt.Printf("Waiting 15 seconds before next HTTPS request...\n")
		time.Sleep(15 * time.Second)
	}
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
