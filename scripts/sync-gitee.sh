#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────
# sync-gitee.sh — 从 GitHub Release 下载安装包并上传到 Gitee
# 用法: GITEE_TOKEN=xxx ./sync-gitee.sh 0.2.3
# ──────────────────────────────────────────────────────────────
set -euo pipefail

VERSION="${1:?用法: $0 <版本号, 如 0.2.3>}"
GITEE_TOKEN="${GITEE_TOKEN:?请设置 GITEE_TOKEN 环境变量}"

GITHUB_REPO="chaoyingLi/files-explorer"
GITEE_REPO="hhyd/files-explorer"
TAG="v${VERSION}"
WORK_DIR="/tmp/gitee-sync-${VERSION}"

# ── 安装包列表 ──
FILES=(
  "Files Explorer_${VERSION}_x64.dmg"
  "Files Explorer_${VERSION}_aarch64.dmg"
  "Files Explorer_${VERSION}_x64_en-US.msi"
  "Files Explorer_${VERSION}_amd64.AppImage"
)

GITHUB_BASE="https://github.com/${GITHUB_REPO}/releases/download/${TAG}"

echo "╔══════════════════════════════════════════════╗"
echo "║  Gitee Release 同步 v${VERSION}                          ║"
echo "╚══════════════════════════════════════════════╝"
echo ""

# ── 1. 获取 Gitee Release ID ──
echo "📡 获取 Gitee Release..."
RESP=$(curl -s -H "Authorization: bearer ${GITEE_TOKEN}" \
  "https://gitee.com/api/v5/repos/${GITEE_REPO}/releases/tags/${TAG}")
RELEASE_ID=$(echo "$RESP" | jq -r '.id // empty')

if [ -z "$RELEASE_ID" ] || [ "$RELEASE_ID" = "null" ]; then
  echo "❌ Gitee Release 不存在，请先在 Actions 中创建"
  echo "   响应: $RESP"
  exit 1
fi
echo "✅ Release ID: $RELEASE_ID"
echo ""

# ── 2. 下载 + 上传 ──
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

OK=0
SKIP=0
FAIL=0

for FILE in "${FILES[@]}"; do
  echo "────────────────────────────────────"
  echo "📥 $FILE"

  # URL 编码空格为 %20
  URL="${GITHUB_BASE}/$(echo "$FILE" | sed 's/ /%20/g')"
  LOCAL_FILE=$(echo "$FILE" | sed 's/ /_/g')

  # 下载（最多重试 3 次）
  if [ -f "$LOCAL_FILE" ]; then
    echo "   ⏭ 已存在，跳过下载"
  else
    echo "   下载中..."
    RETRY=0
    while [ $RETRY -lt 3 ]; do
      if curl -fSL --retry 2 --retry-delay 5 --connect-timeout 30 -# -o "$LOCAL_FILE" "$URL" 2>&1; then
        echo "   ✅ 下载完成 ($(du -h "$LOCAL_FILE" | cut -f1))"
        break
      fi
      RETRY=$((RETRY + 1))
      if [ $RETRY -lt 3 ]; then
        echo "   🔄 重试 $RETRY/3..."
        sleep 5
      else
        echo "   ❌ 下载失败，跳过"
        SKIP=$((SKIP + 1))
      fi
    done
    # 下载失败则跳过上传
    [ ! -f "$LOCAL_FILE" ] && continue
  fi

  # 上传到 Gitee
  echo "   上传到 Gitee..."
  UPLOAD_RESP=$(curl -s -X POST \
    -H "Authorization: bearer ${GITEE_TOKEN}" \
    -F "file=@${LOCAL_FILE}" \
    "https://gitee.com/api/v5/repos/${GITEE_REPO}/releases/${RELEASE_ID}/release_files")

  if echo "$UPLOAD_RESP" | jq -e '.id // empty' > /dev/null 2>&1; then
    echo "   ✅ 上传成功"
    OK=$((OK + 1))
  else
    echo "   ❌ 上传失败: $(echo "$UPLOAD_RESP" | jq -r '.message // "未知错误"')"
    FAIL=$((FAIL + 1))
  fi
done

# ── 3. 清理 ──
cd /tmp
rm -rf "$WORK_DIR"

echo ""
echo "╔══════════════════════════════════════════════╗"
echo "║  完成: 成功 $OK / 跳过 $SKIP / 失败 $FAIL                 ║"
echo "║  Gitee: https://gitee.com/${GITEE_REPO}/releases/${TAG}  ║"
echo "╚══════════════════════════════════════════════╝"
