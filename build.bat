git pull
pnpm install
cd apps\app-frontend
pnpm build
cd ..\app
cargo +stable build --release
cd ..\..
