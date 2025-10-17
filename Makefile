fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings
	# run custom lint rules first
	cargo run --bin lint_rules || exit 1
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test

bench:
	cargo run --bin bench_persist
	cargo run --bin bench_find_similar

ci: fmt lint test
	@echo "CI tasks completed"

# convenience targets for running the demo web server
build:
	cargo build --bin web_server

# choose binary path based on RELEASE flag
BIN := $(if $(filter 1,$(RELEASE)),target/release/web_server,target/debug/web_server)

## start the web server in background, write pid to .tmp/web_server.pid, logs to logs/web_server.log
start: build
	mkdir -p .tmp logs
	@echo "Starting web_server (direct binary): $(BIN)"
	nohup $(BIN) &> logs/web_server.log & echo $$! > .tmp/web_server.pid
	sleep 0.2
	@echo "Started (pid: $$(cat .tmp/web_server.pid))"

stop:
	@echo "Stopping web_server (pidfile -> graceful then kill)";
	@if [ -f .tmp/web_server.pid ]; then \
		pid=$$(cat .tmp/web_server.pid); \
		if [ -n "$$pid" ]; then \
			echo "Found pidfile: $$pid"; \
			kill -TERM $$pid 2>/dev/null || true; \
			sleep 0.5; \
			if kill -0 $$pid 2>/dev/null; then \
				echo "PID $$pid did not exit, sending SIGKILL"; \
				kill -9 $$pid 2>/dev/null || true; \
			fi; \
			rm -f .tmp/web_server.pid; \
		else \
			echo "Pidfile empty"; \
		fi; \
	else \
		echo "No pidfile found (.tmp/web_server.pid)"; \
	fi

restart: stop start

logs:
	@echo "Log file: logs/web_server.log"
	@ls -l logs/web_server.log || true

tail:
	tail -n 200 -f logs/web_server.log
