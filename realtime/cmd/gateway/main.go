package main

import (
	"bufio"
	"encoding/json"
	"log"
	"net"
	"net/http"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/gorilla/websocket"
)

type GpsEvent struct {
	Plate     string    `json:"plate"`
	Latitude  float64   `json:"latitude"`
	Longitude float64   `json:"longitude"`
	Speed     float64   `json:"speed"`
	Source    string    `json:"source"`
	CreatedAt time.Time `json:"created_at"`
}

type Hub struct {
	mu      sync.RWMutex
	clients map[*websocket.Conn]bool
}

func NewHub() *Hub {
	return &Hub{clients: make(map[*websocket.Conn]bool)}
}

func (h *Hub) Add(conn *websocket.Conn) {
	h.mu.Lock()
	defer h.mu.Unlock()
	h.clients[conn] = true
}

func (h *Hub) Remove(conn *websocket.Conn) {
	h.mu.Lock()
	defer h.mu.Unlock()
	delete(h.clients, conn)
	_ = conn.Close()
}

func (h *Hub) Broadcast(event GpsEvent) {
	h.mu.RLock()
	defer h.mu.RUnlock()
	for conn := range h.clients {
		if err := conn.WriteJSON(event); err != nil {
			log.Printf("websocket write failed: %v", err)
		}
	}
}

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

func main() {
	hub := NewHub()

	wsPort := env("REALTIME_WS_PORT", "8090")
	tcpPort := env("REALTIME_TCP_PORT", "7070")
	host := env("REALTIME_HOST", "0.0.0.0")

	go startTCPServer(host+":"+tcpPort, hub)

	mux := http.NewServeMux()
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"status":"ok","service":"go-realtime"}`))
	})
	mux.HandleFunc("/ws", func(w http.ResponseWriter, r *http.Request) {
		handleWebSocket(hub, w, r)
	})
	mux.HandleFunc("/ws/gps", func(w http.ResponseWriter, r *http.Request) {
		handleWebSocket(hub, w, r)
	})

	addr := host + ":" + wsPort
	log.Printf("Go realtime gateway listening on %s", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatalf("http server failed: %v", err)
	}
}

func handleWebSocket(hub *Hub, w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("websocket upgrade failed: %v", err)
		return
	}
	hub.Add(conn)
	defer hub.Remove(conn)

	for {
		if _, _, err := conn.ReadMessage(); err != nil {
			return
		}
	}
}

func startTCPServer(addr string, hub *Hub) {
	listener, err := net.Listen("tcp", addr)
	if err != nil {
		log.Fatalf("tcp server failed: %v", err)
	}
	defer listener.Close()
	log.Printf("GPS TCP socket server listening on %s", addr)

	for {
		conn, err := listener.Accept()
		if err != nil {
			log.Printf("tcp accept failed: %v", err)
			continue
		}
		go handleTCPConnection(conn, hub)
	}
}

func handleTCPConnection(conn net.Conn, hub *Hub) {
	defer conn.Close()
	scanner := bufio.NewScanner(conn)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" {
			continue
		}

		var event GpsEvent
		if err := json.Unmarshal([]byte(line), &event); err != nil {
			log.Printf("invalid gps packet: %v", err)
			continue
		}
		event.Source = "tcp"
		if event.CreatedAt.IsZero() {
			event.CreatedAt = time.Now().UTC()
		}
		hub.Broadcast(event)
	}
}

func env(key, fallback string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return fallback
}
