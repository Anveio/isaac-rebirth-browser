set -euo pipefail
cd /workspace/isaac-rebirth-browser
# This managed executor is non-root and lacks sudo. Install inside its local Docker daemon.
d() {
  env -u DOCKER_HOST -u DOCKER_CONTEXT -u DOCKER_TLS -u DOCKER_TLS_VERIFY -u DOCKER_CERT_PATH docker --host=unix:///var/run/docker.sock "$@"
}
d info >/dev/null
image='node@sha256:3d27e5c11e5786e309ec3e03f93ae536eb36e6e5eb3714d5eb3300a36157add0'
if d container inspect isaac-rebirth-dev >/dev/null 2>&1; then
  d start isaac-rebirth-dev >/dev/null
else
  d pull "$image"
  proxy_ip="$(getent ahostsv4 proxy | awk 'NR==1 {print $1}')"
  test -n "$proxy_ip"
  d run -d --name isaac-rebirth-dev --add-host "proxy:$proxy_ip" \
    -p 127.0.0.1:5173:5173 \
    --mount type=bind,src=/workspace/isaac-rebirth-browser,dst=/workspace/isaac-rebirth-browser \
    --mount type=bind,src=/etc/ssl/certs/ca-certificates.crt,dst=/run/isaac-ca.pem,readonly \
    -e HTTP_PROXY -e HTTPS_PROXY -e NO_PROXY \
    -e http_proxy="$HTTP_PROXY" -e https_proxy="$HTTPS_PROXY" -e no_proxy="$NO_PROXY" \
    -e NODE_EXTRA_CA_CERTS=/run/isaac-ca.pem -e SSL_CERT_FILE=/run/isaac-ca.pem \
    -e REQUESTS_CA_BUNDLE=/run/isaac-ca.pem -e PIP_CERT=/run/isaac-ca.pem \
    -e CURL_CA_BUNDLE=/run/isaac-ca.pem -e APT_CONFIG=/tmp/isaac-apt.conf \
    -w /workspace/isaac-rebirth-browser "$image" sleep infinity
fi
mkdir -p .cache
cat > .cache/cloud-env.sh <<'ENV'
# Supported LLVM 14 Mach-O inspection and Debian LLDB Python module path.
llvm-objdump() {
  if [[ $# == 2 && "$1" == --file-headers && "$2" == *.macho ]]; then
    command llvm-objdump --macho --private-headers "$2"
  else
    command llvm-objdump "$@"
  fi
}
lldb() {
  PYTHONPATH=/usr/lib/llvm-14/lib/python3.11/dist-packages command lldb "$@"
}
export -f llvm-objdump lldb
ENV
proxy_ip="$(getent ahostsv4 proxy | awk 'NR==1 {print $1}')"
# Repair only the session proxy mapping in the restored container.
d exec isaac-rebirth-dev python3 -c 'import sys; p="/etc/hosts"; lines=open(p).readlines(); open(p,"w").writelines((sys.argv[1]+"\tproxy\n") if "proxy" in l.split()[1:] else l for l in lines)' "$proxy_ip"
d exec -i -e HTTP_PROXY -e HTTPS_PROXY -e NO_PROXY -e http_proxy="$HTTP_PROXY" -e https_proxy="$HTTPS_PROXY" -e no_proxy="$NO_PROXY" isaac-rebirth-dev bash <<'INNER'
set -euo pipefail
cd /workspace/isaac-rebirth-browser
printf 'Acquire::https::CaInfo "/run/isaac-ca.pem";\nAcquire::http::Proxy "%s";\nAcquire::https::Proxy "%s";\n' "$HTTP_PROXY" "$HTTPS_PROXY" > /tmp/isaac-apt.conf
source .cache/cloud-env.sh
# CI Playwright requires a free port. Stop only this repository's fixture.
if pgrep -f '^node /workspace/isaac-rebirth-browser/node_modules/.bin/vite --host 0.0.0.0 --port 5173 --strictPort$' >/dev/null; then
  pkill -f '^node /workspace/isaac-rebirth-browser/node_modules/.bin/vite --host 0.0.0.0 --port 5173 --strictPort$'
fi
bash cloud/install.sh
bash cloud/original-install.sh
INNER
