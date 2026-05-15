# EverCore Remote Deploy Packet v0

## Verdict

FLAG: deploy packet is ready for review, but live remote deployment is not proven
until the operator installs the secret env file and runs the NixOS/compose gates.

## Decision

Use the NixOS/workhorse lane for EverCore.

CCR should stay out of the stateful service path. It can still run client-side
smoke tests, propose patches, and review evidence packets.

## Why

- EverCore is a durable memory service; it belongs on the always-on host.
- Hermes can dogfood it locally on the same host through `127.0.0.1`.
- Keeping CCR as a helper preserves the mirror/patch-only boundary.
- Loopback-first exposure prevents the data plane from becoming a public DB
  surface.

## Mutation Boundary

This packet does not mutate the remote host.

Before applying it remotely, confirm:

- the target checkout is clean or intentionally dirty;
- the remote env file exists outside git;
- provider keys are installed only on the host;
- `bindHost` remains `127.0.0.1` unless the operator explicitly approves a
  private network exposure;
- `nixos-rebuild test` passes before `switch`.

## Apply Steps

1. Copy `docker-compose.remote.yaml` and a filled `evercore.env` to the remote
   runtime directory.
2. Put an EverOS checkout at the configured `repoDir`.
3. Import `evercore-remote-workhorse.nix` into the workhorse host config.
4. Run the workhorse rebuild in test mode.
5. Start or restart `evercore-compose.service`.
6. Run `scripts/evercore-remote-smoke.sh --mode health`.
7. After LLM/vector/rerank providers are configured, run
   `scripts/evercore-remote-smoke.sh --mode full`.
8. Point Hermes at `EVEROS_API_BASE_URL=http://127.0.0.1:1995` on the same host,
   or at an operator-controlled private route.

## Red Gates

Keep deployment blocked if any of these are true:

- the API binds to a public interface without explicit approval;
- any data service port is exposed outside Docker/private host boundaries;
- the env file contains placeholder secrets during full smoke;
- `evercore-api` starts without a mounted `/app/.env`;
- health passes but full write/search fails and the provider is marked `PASS`;
- full smoke search returns zero retrievable memories after flush;
- host evidence includes raw public host/IP or credential paths.

## Verification Commands

From this repo:

```bash
bash -n use-cases/hermes-everos-memory/deploy/nixos/scripts/evercore-remote-smoke.sh
EVERCORE_REPO_ROOT=$PWD \
EVERCORE_ENV_FILE=$PWD/use-cases/hermes-everos-memory/deploy/nixos/evercore.env.example \
  docker-compose --env-file use-cases/hermes-everos-memory/deploy/nixos/evercore.env.example \
  -f use-cases/hermes-everos-memory/deploy/nixos/docker-compose.remote.yaml config
```

From the remote host after configuration:

```bash
systemctl status evercore-compose.service
systemctl status evercore-health.timer
scripts/evercore-remote-smoke.sh --mode health
scripts/evercore-remote-smoke.sh --mode full
```

## Next Concrete Action

Stage this packet into the Windburn workhorse lane and run a test rebuild with a
redacted env file present on the host.
