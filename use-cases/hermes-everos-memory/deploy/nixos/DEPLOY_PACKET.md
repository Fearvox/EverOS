# EverCore Remote Deploy Packet v0

## Verdict

FLAG: deploy packet is ready for review and the remote NixOS workhorse is
reachable, but EverCore is not yet running on the workhorse loopback service.

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
- DeepSeek/OpenRouter provider key is installed only on the host;
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
7. After DeepSeek/OpenRouter LLM auth plus vector/rerank providers are
   configured, run
   `scripts/evercore-remote-smoke.sh --mode full`.
8. Point Hermes at `EVEROS_API_BASE_URL=http://127.0.0.1:1995` on the same host,
   or at an operator-controlled private route.

## Red Gates

Keep deployment blocked if any of these are true:

- the API binds to a public interface without explicit approval;
- any data service port is exposed outside Docker/private host boundaries;
- the env file contains placeholder secrets during full smoke;
- DeepSeek/OpenRouter auth preflight fails;
- `evercore-api` starts without a mounted `/app/.env`;
- health passes but full write/search fails and the provider is marked `PASS`;
- full smoke search returns zero retrievable memories after flush;
- host evidence includes raw public host/IP or credential paths.

## Observed Remote Probe

Latest read-only probe:

- remote host is reachable through the existing workhorse SSH route;
- remote OS is NixOS and system state is running;
- failed systemd units reported as zero during the dry-run NixOS probe;
- `evercore-compose.service` is inactive;
- `evercore-health.timer` is inactive;
- `http://127.0.0.1:1995/health` is unavailable on the remote host.

Verdict: `FLAG`, because the target host is real and healthy enough for deploy
work, but EverCore has not been applied or started there.

## Verification Commands

From this repo:

```bash
bash -n use-cases/hermes-everos-memory/deploy/nixos/scripts/evercore-remote-smoke.sh
cd use-cases/hermes-everos-memory && just deepseek-auth-preflight
EVERCORE_REPO_ROOT=$PWD \
EVERCORE_ENV_FILE=$PWD/use-cases/hermes-everos-memory/deploy/nixos/evercore.env.example \
  docker-compose --env-file use-cases/hermes-everos-memory/deploy/nixos/evercore.env.example \
  -f use-cases/hermes-everos-memory/deploy/nixos/docker-compose.remote.yaml config
```

From the remote host after configuration:

```bash
repo/use-cases/hermes-everos-memory/scripts/deepseek-auth-preflight.sh --env evercore.env --require-key
systemctl status evercore-compose.service
systemctl status evercore-health.timer
scripts/evercore-remote-smoke.sh --mode health
scripts/evercore-remote-smoke.sh --mode full
```

## Next Concrete Action

Repair the runtime lane by using the DeepSeek/OpenRouter auth path, prove it
with `deepseek-auth-preflight.sh --require-key`, then resume the guarded
`nixos-rebuild test` path. Keep `switch` blocked until `test` passes.
