# EverCore Remote On NixOS

This packet deploys EverCore as a remote memory backend for Hermes without
turning CCR into the stateful service host.

Decision:

- NixOS/workhorse owns the long-running EverCore service.
- CCR remains a client/helper lane for patch proposals, smoke tests, and review.
- Public data ports stay closed. Hermes should reach EverCore through same-host
  loopback, VPN, or an operator-controlled tunnel.

## Files

| File | Purpose |
| --- | --- |
| `docker-compose.remote.yaml` | EverCore API plus MongoDB, Redis, Elasticsearch, MinIO, and Milvus |
| `evercore.env.example` | Sanitized remote env template; copy to `evercore.env` outside git |
| `evercore-remote-workhorse.nix` | Optional NixOS module for the workhorse |
| `scripts/evercore-remote-smoke.sh` | Public-safe health/write/search smoke helper |

## Security Contract

- Do not expose MongoDB, Redis, Elasticsearch, MinIO, or Milvus on public
  interfaces.
- Do not commit `evercore.env`, provider keys, SSH targets, raw host values, or
  local credential paths.
- Keep the API bound to `127.0.0.1` unless the host is behind a private network
  route and the operator explicitly opens it.
- Run `nixos-rebuild test` before `switch` when this module is imported into a
  live Windburn host.

## Remote Layout

Recommended host layout:

```text
/srv/windburn/evercore/
  docker-compose.remote.yaml
  evercore.env
  repo/                  # EverOS checkout, or symlink to the checkout
  backups/
```

The compose file expects:

- `EVERCORE_REPO_ROOT` to point at the EverOS checkout root.
- `EVERCORE_ENV_FILE` to point at the secret-bearing env file.
- `EVERCORE_BIND_HOST` and `EVERCORE_BIND_PORT` to control API exposure.

The default bind is `127.0.0.1:1995`.

## Manual Bring-Up

On the remote host:

```bash
cp evercore.env.example evercore.env
$EDITOR evercore.env

export EVERCORE_REPO_ROOT=/srv/windburn/evercore/repo
export EVERCORE_ENV_FILE=/srv/windburn/evercore/evercore.env
export EVERCORE_BIND_HOST=127.0.0.1
export EVERCORE_BIND_PORT=1995

docker-compose --env-file "$EVERCORE_ENV_FILE" \
  -f docker-compose.remote.yaml up -d --build
```

Health-only smoke:

```bash
EVEROS_API_BASE_URL=http://127.0.0.1:1995 \
  scripts/evercore-remote-smoke.sh --mode health
```

Full memory smoke, after LLM/vector/rerank providers are configured:

```bash
EVEROS_API_BASE_URL=http://127.0.0.1:1995 \
  scripts/evercore-remote-smoke.sh --mode full
```

`full` mode checks health, writes one agent-memory turn, flushes the session,
then blocks if search returns no retrievable memory.

## NixOS Bring-Up

Copy `evercore-remote-workhorse.nix` into the workhorse module set, import it in
the host configuration, and override at least these options:

```nix
services.evercoreRemote = {
  enable = true;
  baseDir = "/srv/windburn/evercore";
  repoDir = "/srv/windburn/evercore/repo";
  envFile = "/srv/windburn/evercore/evercore.env";
  composeFile = "/srv/windburn/evercore/docker-compose.remote.yaml";
  bindHost = "127.0.0.1";
  bindPort = 1995;
};
```

Keep `openFirewall = false` for v0.

## Hermes Provider

For Hermes running on the same remote host:

```bash
export EVEROS_API_BASE_URL=http://127.0.0.1:1995
export EVEROS_USER_ID=hermes-user
export EVEROS_AGENT_ID=hermes
```

For local Hermes talking to remote EverCore, use a private route or tunnel and
keep the provider config pointed at the local endpoint exposed by that route.

## Gates

`PASS` for deploy readiness requires:

1. `docker-compose ps` shows every service healthy.
2. `scripts/evercore-remote-smoke.sh --mode health` passes.
3. `scripts/evercore-remote-smoke.sh --mode full` passes after provider keys are
   installed.
4. Hermes provider `everos_health`, `everos_store`, and `everos_search` all pass.
5. No public data ports are reachable from outside the private host boundary.
