FROM debian:stable-slim
COPY sentinelle /usr/local/bin/sentinelle
ENTRYPOINT ["/bin/sh","-c","while true; do /usr/local/bin/sentinelle check \"$@\"; sleep 60; done","--"]
