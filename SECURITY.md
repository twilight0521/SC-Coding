# 安全规范

## 1. 安全目标

SuperCompany Coding 会访问用户代码、本地文件、API Key 和终端命令，因此安全优先级必须高于功能速度。

## 2. API Key 安全

### 2.1 存储

优先使用：

- macOS Keychain
- Windows Credential Manager

备用方案：

- 本地加密文件
- 主密钥由系统安全存储生成

禁止：

- localStorage
- 明文 JSON
- 明文 SQLite
- 明文日志

### 2.2 展示

API Key 展示规则：

```text
sk-1234************abcd
```

只允许用户重新填写，不允许完整查看。

### 2.3 导出

导出 Provider 配置时，默认不导出 API Key。

## 3. 文件安全

以下文件默认不发送给模型：

```text
.env
.env.*
*.pem
*.key
id_rsa
id_ed25519
*.p12
*.pfx
credentials.json
service-account*.json
node_modules/
dist/
build/
.git/
```

## 4. 命令安全

### 4.1 命令等级

| 等级 | 示例 | 策略 |
|---|---|---|
| 安全 | npm test、npm run build | 可自动执行 |
| 中风险 | npm install、pip install | 首次确认 |
| 高风险 | rm -rf、sudo、curl pipe sh | 默认禁止 |

### 4.2 默认禁止命令

```text
rm -rf
sudo
curl * | sh
wget * | sh
chmod -R 777
chown -R
mkfs
ssh
scp
rsync --delete
git push --force
npm publish
pnpm publish
```

## 5. Agent 权限

Agent 默认最小权限：

- Orchestrator：只读，不写文件，不执行命令。
- Coder：可写项目文件，不执行命令。
- Tester：可执行测试命令，不安装依赖。
- Debugger：可写文件，可执行安全命令。
- Security：只读。

## 6. 上下文发送预览

高级模式下，用户可以查看：

- 哪些文件会发送给模型。
- 哪些内容被脱敏。
- 发送给哪个 Provider。
- 预计 Token 数。

## 7. 日志脱敏

日志中必须脱敏：

- API Key
- Authorization Header
- Cookie
- Token
- 私钥
- .env 内容

## 8. 网络安全

默认不允许 Agent 随意访问网络。

需要联网时必须说明：

- 访问目的
- 访问域名
- 请求方式
- 是否下载文件
- 是否执行下载内容

## 9. 模型侧隐私策略

Provider 配置页需要展示：

- 当前模型服务商
- 数据会发送到哪里
- 是否第三方 API
- 是否本地模型

用户可设置项目级策略：

1. 只允许本地模型。
2. 只允许指定 Provider。
3. 禁止发送指定目录。
4. 禁止发送超过 N 行的文件。

## 10. 安全验收标准

1. API Key 不明文落盘。
2. 日志不包含 API Key。
3. .env 默认不发送给模型。
4. 高风险命令必须拦截。
5. 删除文件必须用户确认。
6. Provider 导出不包含 API Key。
7. Agent 权限可配置。
8. 用户可以查看模型调用历史。
