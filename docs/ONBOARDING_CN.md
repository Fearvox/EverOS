# EverOS 新手入门指南

欢迎加入 EverOS 团队！这份指南会用最简单的方式帮你跑起来。

---

## 这个项目是干啥的？

**一句话**：给 AI Agent 加上长期记忆，让它能记住你说过的话。

想象一下：你每次和 ChatGPT 聊天，它都不记得上次聊了什么。EverOS 就是解决这个问题的——让 Agent 有记忆力。

---

## 项目结构（只看重点）

```text
EverOS/
├── methods/EverCore/     <-- 核心代码！主要看这里
├── benchmarks/           <-- 跑分测试
├── use-cases/            <-- 应用案例
└── docs/                 <-- 文档
```

**新人只需要关心 `methods/EverCore/`**，其他的以后再说。

---

## 5 分钟跑起来

### 前置条件

确保你电脑上有：

- Docker（不知道有没有？跑 `docker --version`）
- Python 3.10+（跑 `python --version`）

没有的话先装：

- Docker: <https://docs.docker.com/get-docker/>
- Python: <https://www.python.org/downloads/>

### 第 1 步：进入核心目录

```bash
cd methods/EverCore
```

### 第 2 步：启动数据库们

```bash
docker compose up -d
```

这一步会启动 MongoDB、Redis、Elasticsearch、Milvus。等 1-2 分钟。

检查是否启动成功：

```bash
docker compose ps
```

看到都是 `Up` 或 `running` 就对了。

### 第 3 步：安装 Python 依赖

```bash
# 先装 uv（如果没有）
curl -LsSf https://astral.sh/uv/install.sh | sh

# 装依赖
uv sync
```

### 第 4 步：配置环境变量

```bash
cp env.template .env
```

然后编辑 `.env` 文件，填上你的 API Key：

```text
OPENAI_API_KEY=sk-xxx你的key
```

### 第 5 步：跑起来

```bash
uv run python src/run.py
```

看到 `http://localhost:1995` 就成功了！

---

## 常用命令速查

| 做什么 | 命令 |
|--------|------|
| 启动服务 | `docker compose up -d` |
| 停止服务 | `docker compose down` |
| 跑程序 | `uv run python src/run.py` |
| 跑测试 | `make test` |
| 代码格式化 | `make lint` |

---

## 代码在哪看？

核心代码都在 `methods/EverCore/src/`：

```text
src/
├── run.py                          <-- 启动入口
├── agentic_layer/
│   └── memory_manager.py           <-- 记忆管理（重要！）
├── memory_layer/
│   └── prompts/                    <-- 提示词
└── infra_layer/
    └── adapters/input/api/         <-- REST API
```

**建议先看**：

1. `src/run.py` - 程序从哪里开始
2. `src/agentic_layer/memory_manager.py` - 记忆是怎么管理的

---

## 遇到问题？

### Docker 起不来

```bash
docker compose down
docker compose up -d
```

### 端口被占用

换个端口，在 `.env` 里改 `PORT=其他端口号`

### 依赖装不上

```bash
uv cache clean
uv sync --refresh
```

---

## 下一步

1. 看看 [EverCore 文档](../methods/EverCore/docs/)
2. 跑跑 [Demo 示例](../methods/EverCore/demo/)
3. 有问题随时问团队！

---

**记住**：不懂就问，没人会笑话你。我们都是从新手过来的。
