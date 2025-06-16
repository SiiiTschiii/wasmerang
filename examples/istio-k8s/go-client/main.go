package main

import (
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
)

func main() {
	// Test both HTTP and HTTPS requests
	go testHTTPConnection()
	go testHTTPSConnection()
	
	// Keep the main goroutine alive
	select {}
}

func testHTTPConnection() {
	url := "http://httpbin.org/ip"
	
	for {
		resp, err := http.Get(url)
		if err != nil {
			fmt.Printf("HTTP httpbin.org - ERROR: %v\n", err)
		} else {
			io.ReadAll(resp.Body)
			resp.Body.Close()
			fmt.Printf("HTTP httpbin.org - %d\n", resp.StatusCode)
		}
		
		time.Sleep(3 * time.Second)
	}
}

func testHTTPSConnection() {
	url := "https://httpbin.org/ip"
	
	for {
		resp, err := http.Get(url)
		if err != nil {
			fmt.Printf("HTTPS httpbin.org - ERROR: %v\n", err)
		} else {
			body, _ := io.ReadAll(resp.Body)
			resp.Body.Close()
			
			response := string(body)
			if strings.Contains(response, "\"origin\":") {
				// Extract the IP from the JSON response
				start := strings.Index(response, "\"origin\": \"") + 11
				end := strings.Index(response[start:], "\"")
				if end > 0 {
					ip := response[start : start+end]
					fmt.Printf("HTTPS httpbin.org %s %d\n", ip, resp.StatusCode)
				} else {
					fmt.Printf("HTTPS httpbin.org - %d\n", resp.StatusCode)
				}
			} else {
				fmt.Printf("HTTPS httpbin.org - %d\n", resp.StatusCode)
			}
		}
		
		time.Sleep(3 * time.Second)
	}
}
