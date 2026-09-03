# Tutorial 8: Manage users and roles (advanced)

This advanced tutorial demonstrates Milvus RBAC with the Rust SDK. It creates a temporary user,
role, and privilege group, grants the role to the user, inspects the resulting permissions, and
then revokes and removes everything it created.

## Prerequisites

- Rust and Cargo are installed.
- Milvus 2.6 or later is running.
- The connection in `MILVUS_TOKEN` has permission to manage users and roles when authorization is
  enabled.
- `milvus-sdk-rust` version `2.6.1` has been published to crates.io.

Connection settings use `MILVUS_URI` and `MILVUS_TOKEN`, defaulting to
`http://localhost:19530` and `root:Milvus`.

## Run

```bash
cargo run --manifest-path tutorial/8_rbac/Cargo.toml
```

The tutorial uses unique names so it can be run repeatedly. On normal completion it attempts every
teardown step for its user, role, grants, and privilege group. If any cleanup operation fails, it
reports all collected cleanup errors and exits unsuccessfully rather than silently leaving
server-level RBAC resources behind.

This tutorial does not require the repository's standalone server to enable
`common.security.authorizationEnabled=true`. With authorization disabled, Milvus may accept the
RBAC management calls, but it does not enforce the temporary user's `Query`/`Search` permissions;
the example therefore cannot validate that an unauthorized insert is rejected. To test actual
permission enforcement, run the same tutorial against a separately configured authorization-enabled
Milvus instance and provide an administrative `MILVUS_TOKEN`.

## Expected output

Names are unique, but the run should include messages like:

```text
Calling create_user: create "u<PID>_<MILLIS>"
create_user completed
Calling grant_role: assign "rust_tutorial_role_..." to "u<PID>_<MILLIS>"
grant_role completed
Calling drop_privilege_group: remove "rust_tutorial_group_..."
drop_privilege_group completed: ok
```

## Troubleshooting

- RBAC management errors usually mean the token lacks administrative privileges.
- If authorization is disabled, a successful insert by the temporary user is expected and does not
  prove that the grants were enforced.
- `connection refused` means Milvus is not running at `MILVUS_URI`.
