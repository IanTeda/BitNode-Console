## Structure

This document describes the structure of the BitNode Console project.

## Directory Structure

```bash
.
├── Cargo.toml                          # Workspace root
├── Makefile.toml                       # cargo-make task definitions
├── rustfmt.toml                        # Rust formatter configuration
├── shell.nix                           # Nix development environment
├── bitnode_console.conf                # Local development config
├── book.toml                           # mdBook configuration
├── mkdocs.yaml                         # MkDocs configuration
│
├── backend/
│   ├── app/                            # Application binary crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs                 # Entry point
│   │   │   ├── lib.rs
│   │   │   └── error.rs
│   │   └── tests/
│   │       ├── rpc/                    # RPC integration tests
│   │       └── web/                    # Web integration tests
│   │
│   └── libs/
│       ├── lib_auth/                   # Authentication library
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── error.rs
│       │       └── domains.rs          # PasswordHash domain type
│       │
│       ├── lib_rpc/                    # gRPC server library
│       │   ├── Cargo.toml
│       │   ├── build.rs                # Protobuf code generation
│       │   ├── protos/
│       │   │   └── bitnode_console/v1/
│       │   │       └── utilities.proto
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── error.rs
│       │       ├── server.rs
│       │       ├── generated_protos/   # Generated protobuf code
│       │       └── services/
│       │           └── utilities.rs
│       │
│       ├── lib_settings/               # Configuration parsing library
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── error.rs
│       │       ├── settings.rs         # Multi-source config parsing
│       │       ├── application.rs      # Application settings
│       │       ├── rpc.rs              # RPC settings
│       │       ├── tracing.rs          # Tracing settings
│       │       └── web.rs              # Web server settings
│       │
│       ├── lib_tracing/                # Tracing/logging library
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── error.rs
│       │       ├── domain.rs
│       │       └── init.rs
│       │
│       └── lib_web/                    # HTTP server library
│           ├── Cargo.toml
│           ├── build.rs                # Frontend embedding
│           └── src/
│               ├── lib.rs
│               ├── error.rs
│               └── server.rs
│
├── frontend/                           # React + TypeScript SPA
│   ├── package.json
│   ├── vite.config.ts                  # Vite bundler configuration
│   ├── tailwind.config.ts              # Tailwind CSS configuration
│   ├── components.json                 # shadcn/ui configuration
│   ├── tsconfig.json
│   ├── tsconfig.app.json
│   ├── tsconfig.node.json
│   ├── eslint.config.js
│   ├── index.html
│   └── src/
│       ├── main.tsx                    # Application entry point
│       ├── index.css                   # Global styles
│       ├── routeTree.gen.ts            # Generated route tree
│       ├── assets/                     # Static assets
│       ├── components/
│       │   ├── layouts/                # Layout components
│       │   │   ├── AuthLayout.tsx
│       │   │   ├── PublicLayout.tsx
│       │   │   └── RestrictedLayout.tsx
│       │   ├── ui/                     # shadcn/ui components
│       │   ├── ErrorBoundary.tsx
│       │   ├── TanStackDevTools.tsx
│       │   ├── logged-out-card.tsx
│       │   └── login-form.tsx
│       ├── hooks/
│       │   └── use-theme.ts
│       ├── lib/
│       │   ├── logger.ts
│       │   └── utils.ts
│       └── routes/
│           ├── __root.tsx              # Root route layout
│           ├── index.tsx               # Index redirect
│           ├── _public/
│           │   └── auth/               # Public auth pages
│           │       ├── route.tsx
│           │       ├── login.tsx
│           │       ├── logged-out.tsx
│           │       ├── verifying.tsx
│           │       └── $.tsx           # Auth catch-all redirect
│           └── _restricted/            # Authenticated pages
│               ├── route.tsx
│               ├── dashboard.tsx
│               ├── logs.tsx
│               ├── network.tsx
│               ├── node.tsx
│               ├── settings.tsx
│               └── $.tsx               # Restricted catch-all redirect
│
└── docs/                               # Project documentation
    ├── SUMMARY.md
    ├── index.md
    ├── product-requirements.md
    ├── configuration.md
    ├── routing.md
    ├── structure.md
    ├── css/
    └── images/
```

## Workspace Crate Dependency Graph

```text
app
├── lib_auth        # Authentication (Argon2 password hashing)
├── lib_rpc         # gRPC server (tonic + protobuf)
├── lib_settings    # Configuration parsing (config-rs)
├── lib_tracing     # Tracing/logging initialisation
└── lib_web         # HTTP server (axum, serves frontend SPA)
```
