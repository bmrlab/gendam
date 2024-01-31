This is a [Next.js](https://nextjs.org/) project bootstrapped with [`create-next-app`](https://github.com/vercel/next.js/tree/canary/packages/create-next-app).

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

You can start editing the page by modifying `app/page.tsx`. The page auto-updates as you edit the file.

This project uses [`next/font`](https://nextjs.org/docs/basic-features/font-optimization) to automatically optimize and load Inter, a custom Google Font.

## Prisma Rust Client

1. 添加 `prisma-client-rust` 和 `prisma-client-rust-cli` 两个 crate
2. 添加 `bin/prisma.rs` 并在 `main` 中执行 `prisma_client_rust_cli::run();`, 搞定 prisma cli

```bash
cd src-tauri
cargo run --bin prisma
# or
cargo run --bin prisma -- <command>
```

为了方便使用，可以在 `.cargo/config.toml` 中添加一个 alias

```toml
[alias]
prisma = "run --bin prisma --"
```

3. 执行 `cargo prisma init` 初始化 prisma 配置, 这时候会在 `src-tauri` 下生成一个 `prisma` 目录, 接着需要把 schema.prisma 里面的 client 配置修改成如下

```
generator client {
  provider = "cargo prisma"
  output = "src/prisma/mod.rs"
}
```

4. 执行 `cargo prisma generate` 生成 artifacts (e.g. Prisma Client), 根据上一步配置, 生成在 `src/prisma` 目录下

5. `cargo prisma migrate dev` 以后，在代码里直接引入 `PrismaClient` 开始使用
