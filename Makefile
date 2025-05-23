APP_NAME := patreon-reader-service
PORT := 8080

.PHONY: all

all:
	docker build -t $(APP_NAME) . && docker run -p $(PORT):$(PORT) $(APP_NAME)

