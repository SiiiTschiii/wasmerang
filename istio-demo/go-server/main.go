package main

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"time"
)

func main() {
	name := os.Getenv("SERVER_NAME")
	if name == "" {
		name = "unknown-server"
	}
	
	// Create a logging handler that wraps the main handler
	loggingHandler := func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		
		// Log detailed request information
		log.Printf("[%s] 🎯 INTERCEPTED REQUEST: %s %s", name, r.Method, r.URL.Path)
		log.Printf("[%s] 📡 Request from: %s", name, r.RemoteAddr)
		log.Printf("[%s] 🌐 Host header: %s", name, r.Host)
		log.Printf("[%s] 🔗 User-Agent: %s", name, r.UserAgent())
		
		// Handle the request
		response := fmt.Sprintf("Hello from %s\n", name)
		w.Header().Set("X-Egress-Server", name)  // Add header to identify which server handled it
		fmt.Fprint(w, response)
		
		// Log completion
		duration := time.Since(start)
		log.Printf("[%s] ✅ Response sent in %v", name, duration)
		log.Printf("[%s] ==========================================", name)
	}
	
	http.HandleFunc("/", loggingHandler)
	
	// Start server on port 8080
	go func() {
		log.Printf("[%s] 🚀 Starting HTTP server on port 8080", name)
		if err := http.ListenAndServe(":8080", nil); err != nil {
			log.Printf("[%s] ❌ Error on port 8080: %v", name, err)
		}
	}()
	
	// Start server on port 8081
	log.Printf("[%s] 🚀 Starting HTTP server on port 8081", name)
	if err := http.ListenAndServe(":8081", nil); err != nil {
		log.Printf("[%s] ❌ Error on port 8081: %v", name, err)
	}
}
