# syntax=docker/dockerfile:1
FROM node:24-bookworm
WORKDIR /workspace/isaac-rebirth-browser
COPY . .
# Optional combined public CA bundle for managed-cloud proxy trust. Supply with
# --secret id=proxy_ca,src=/etc/ssl/certs/ca-certificates.crt in that executor.
RUN --mount=type=secret,id=proxy_ca \
    if [ -f /run/secrets/proxy_ca ]; then \
      export NODE_EXTRA_CA_CERTS=/run/secrets/proxy_ca; \
      export PIP_CERT=/run/secrets/proxy_ca; \
      export SSL_CERT_FILE=/run/secrets/proxy_ca; \
      export CURL_CA_BUNDLE=/run/secrets/proxy_ca; \
      export APT_CONFIG=/tmp/isaac-proxy-apt.conf; \
      printf 'Acquire::https::CaInfo "/run/secrets/proxy_ca";\n' > "$APT_CONFIG"; \
    fi; \
    bash cloud/install.sh
EXPOSE 5173
CMD ["npm", "run", "dev"]

