#!/usr/bin/env bash

export METRICS_PROXY_HOST=${METRICS_PROXY_HOST:-0.0.0.0}
export METRICS_PROXY_PORT=${METRICS_PROXY_PORT:-8080}

gomplate -f /app/configs/config.template > /app/configs/config.toml

export PROXY_CONFIG_FILE=/app/configs/config.toml

case $1 in
    bash)
        bash
        ;;
    dummy)
        # Dummy section to debug
        while true; do sleep 30; echo "ping"; done;
        ;;
    metrics-proxy)
        echo "Starting the metrics-proxy..."
        /app/metrics-proxy
        ;;
    help|*)
        echo "metrics-proxy"
        echo ""
        echo "USAGE:"
        echo "  docker-entrypoint [SUBCOMMAND]"
        echo ""
        echo "SUBCOMMANDS:"
        echo "  help    Prints this message"
        echo "  run     Runs metrics-prxy"
        echo "  bash    Runs bash"
        echo "  dummy   Sleeps for 30 seconds and print 'ping'"
esac
