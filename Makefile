.PHONY: help build run stop restart logs status

IMAGE  = log-server-rust:latest
NAME   = log-server
PORT  ?= 9006

help:
	@echo "Log Service"
	@echo ""
	@echo "  make build    - Build Docker image"
	@echo "  make run      - Start container (builds first if needed)"
	@echo "  make stop     - Stop and remove container"
	@echo "  make restart  - Stop, rebuild, and start"
	@echo "  make logs     - Tail container logs"
	@echo "  make status   - Show container status"

build:
	@echo "Building $(IMAGE)..."
	@docker build -t $(IMAGE) server

run: build
	@if docker ps -q -f name=$(NAME) | grep -q .; then \
		echo "$(NAME) is already running"; \
	else \
		docker rm -f $(NAME) 2>/dev/null || true; \
		echo "Starting $(NAME) on port $(PORT)..."; \
		docker run -d --name $(NAME) -p $(PORT):9006 $(IMAGE); \
	fi

stop:
	@docker rm -f $(NAME) 2>/dev/null && echo "Stopped $(NAME)" || echo "$(NAME) is not running"
	@sleep 1

restart: stop build
	@echo "Starting $(NAME) on port $(PORT)..."
	@docker run -d --name $(NAME) -p $(PORT):9006 $(IMAGE)

logs:
	@docker logs -f $(NAME)

status:
	@docker ps -f name=$(NAME) --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || echo "$(NAME) is not running"
