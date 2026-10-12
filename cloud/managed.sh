#!/usr/bin/env bash
set -euo pipefail
d() { env -u DOCKER_HOST -u DOCKER_CONTEXT -u DOCKER_TLS -u DOCKER_TLS_VERIFY -u DOCKER_CERT_PATH docker --host=unix:///var/run/docker.sock "$@"; }
d start isaac-rebirth-dev >/dev/null
proxy_ip="$(getent ahostsv4 proxy | awk 'NR==1 {print $1}')"
test -n "$proxy_ip"
d exec isaac-rebirth-dev /usr/bin/python3 -c 'import sys; p="/etc/hosts"; lines=open(p).readlines(); open(p,"w").writelines((sys.argv[1]+"\tproxy\n") if "proxy" in l.split()[1:] else l for l in lines)' "$proxy_ip"
exec env -u DOCKER_HOST -u DOCKER_CONTEXT -u DOCKER_TLS -u DOCKER_TLS_VERIFY -u DOCKER_CERT_PATH docker --host=unix:///var/run/docker.sock exec \
  -e HTTP_PROXY -e HTTPS_PROXY -e NO_PROXY \
  -e http_proxy="$HTTP_PROXY" -e https_proxy="$HTTPS_PROXY" -e no_proxy="$NO_PROXY" \
  -e PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  isaac-rebirth-dev "$@"
