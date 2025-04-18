# Janus 進程管理器

Janus 是一個輕量級的進程管理器，專為容器環境設計，能夠管理多個進程的生命週期。它使用 Rust 語言開發，具有高效能、低資源消耗和強大的錯誤處理能力。

## 主要功能

- 進程生命週期管理（啟動、停止、重啟）
- 配置驅動的進程定義
- 進程監控與自動重啟
- 日誌統一管理
- 優雅的信號處理
- 簡潔的命令行介面

## 快速開始

### 安裝

```bash
# 從源碼編譯
git clone https://github.com/yourusername/janus.git
cd janus
cargo build --release
```

### 基本使用

1. 創建配置文件 `janus.toml`:

```toml
# 全局配置
[global]
working_dir = "/app"
log_level = "info"

# 環境變量
[global.env]
TZ = "Asia/Shanghai"

# 進程定義
[[process]]
name = "web-server"
command = "nginx"
args = ["-g", "daemon off;"]
auto_restart = true

[[process]]
name = "app-server"
command = "node"
args = ["server.js"]
working_dir = "/app/server"
auto_restart = true
restart_limit = 3
restart_delay = 5
```

2. 啟動所有進程:

```bash
janus --config janus.toml start
```

3. 檢查進程狀態:

```bash
janus --config janus.toml status
```

## 命令參考

```
janus [OPTIONS] COMMAND [ARGS]...

OPTIONS:
  -c, --config FILE    指定配置文件路徑
  --help               顯示幫助信息
  --version            顯示版本信息

COMMANDS:
  start                啟動所有進程
  stop                 停止所有進程
  restart              重啟所有進程
  status               顯示進程狀態
  start-one NAME       啟動單個進程
  stop-one NAME        停止單個進程
  restart-one NAME     重啟單個進程
```

## 配置文件說明

### 全局配置

| 選項 | 類型 | 必填 | 描述 |
|------|------|------|------|
| working_dir | String | 否 | 默認工作目錄 |
| log_level | String | 否 | 日誌級別 (debug/info/warn/error) |
| env | Map | 否 | 全局環境變量 |

### 進程配置

| 選項 | 類型 | 必填 | 描述 |
|------|------|------|------|
| name | String | 是 | 進程名稱 (唯一) |
| command | String | 是 | 執行命令 |
| args | String[] | 否 | 命令參數 |
| working_dir | String | 否 | 工作目錄 (覆蓋全局) |
| env | Map | 否 | 環境變量 (合併全局) |
| auto_restart | Boolean | 否 | 是否自動重啟 (默認 false) |
| restart_limit | Integer | 否 | 最大重啟次數 (默認無限) |
| restart_delay | Integer | 否 | 重啟延遲秒數 (默認 1) |

## 容器化使用

Janus 特別適合在容器環境中使用，作為容器的入口點管理多個進程：

```dockerfile
FROM your-base-image

# 安裝 Janus
COPY --from=janus:latest /usr/local/bin/janus /usr/local/bin/

# 添加配置文件
COPY janus.toml /app/janus.toml

# 設置入口點
ENTRYPOINT ["janus", "--config", "/app/janus.toml", "start"]
```

## 使用案例

### 作為容器入口點

在 Docker 容器中管理多個服務：

```toml
[[process]]
name = "nginx"
command = "nginx"
args = ["-g", "daemon off;"]
auto_restart = true

[[process]]
name = "php-fpm"
command = "php-fpm"
args = ["--nodaemonize"]
auto_restart = true
```

### 開發環境

在開發環境中同時啟動多個服務：

```toml
[[process]]
name = "frontend"
command = "npm"
args = ["run", "dev"]
working_dir = "/app/frontend"
auto_restart = true

[[process]]
name = "backend"
command = "npm"
args = ["run", "dev"]
working_dir = "/app/backend"
auto_restart = true

[[process]]
name = "cache"
command = "redis-server"
auto_restart = true
```

## 下載

您可以從以下連結下載 Janus 進程管理器：

- [GitHub Releases](https://github.com/yourusername/janus/releases)
- [Crates.io](https://crates.io/crates/janus)

或者使用 Cargo 直接安裝：

```bash
cargo install janus
```

## 貢獻

歡迎提交問題報告和功能請求！如果您想貢獻代碼，請遵循以下步驟：

1. Fork 本項目
2. 創建您的特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交您的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 開啟一個 Pull Request

## 授權

本項目採用 MIT 授權 - 詳見 [LICENSE](LICENSE) 文件。