#!/usr/bin/env bash
# bump-version.sh — 统一修改所有版本号
# 用法: ./scripts/bump-version.sh 0.3.0
set -euo pipefail
V="${1:?用法: $0 <新版本号>}"

jq ".version = \"$V\"" src-tauri/tauri.conf.json > tmp.json && mv tmp.json src-tauri/tauri.conf.json
jq ".version = \"$V\"" package.json > tmp.json && mv tmp.json package.json
sed -i '' "s/^version = .*/version = \"$V\"/" src-tauri/Cargo.toml
echo "✅ 版本号 → $V"
echo "⚠️  updater.json 已废弃，由 Tauri createUpdaterArtifacts 自动生成 latest.json"
