# 此处作为任务进行保存点,方便不同ai写作

现在正在进行ui重构

# LLM 学习工作区实施批次

本文档将已批准的工程蓝图拆分为实施批次。每个批次是一个人工检查点，在人类审阅并接受当前批次之前，不要开始下一批次。

## 全局 UI 语言规则

前端界面使用中文展示

## 项目目标

- Rust 后端核心。
- Vue 前端。
- PostgreSQL 数据库。
- 工作区条目模型。
- Tiptap/ProseMirror 结构化笔记。
- KaTeX 数学渲染。
- 像工作区文档一样打开的 LLM 聊天窗格。
- 用户自定义智能体。
- 可手动编辑的 OpenAI 兼容请求 JSON。
- 提示块。
- 文件工具、调度器和写入审计稍后添加。

## 所有批次的全局规则

- 保持 Rust 作为唯一的可信后端。前端不得直接访问数据库、本地文件系统或 LLM 提供商。
- 保持每个持久化形态显式并带有迁移支持。
- 优先使用可以在 UI 中人工检查的小型垂直切片。
- 每个涉及数据库变更的批次必须包含迁移和可回滚的测试。
- 每个 API 批次必须包含一致的错误响应。
- 每个流式传输或智能体批次必须持久化足够的事件以供回放/调试。
- 每个前端批次必须暴露足够的 UI 以验证后端行为。
- 除非后续计划明确将以下内容移出推迟范围，否则不要添加登录/认证、向量检索、全文搜索或多用户逻辑。
- 避免硬编码的智能体类型。所有智能体行为必须来自用户配置。
- 不要将 Markdown 视为笔记的来源真值。笔记以 Tiptap/ProseMirror JSON 存储。


### 工作台 UI 规则

- IDE 风格的工作台外壳是第一方 UI，而非第三方视觉组件库皮肤。
- 仅将 Reka UI 用作无头交互层，用于非平凡行为，如下拉菜单、对话框、弹出框、tooltip、标签页、焦点管理和键盘导航。
- 在从功能视图使用 Reka 原语之前，将其包装在 `frontend/src/ui` 下的本地 `Wb*` 组件中。
- 功能视图应使用 `WbButton`、`WbIconButton` 以及未来的 `WbSelect`、`WbDialog`、`WbPopover`、`WbTree` 和 `WbTabs`，而不是直接导入 Reka UI 或带样式的第三方组件。
- 视觉一致性由 `frontend/src/ui/tokens.css` 和 `frontend/src/ui/components.css` 管理；新颜色、间距、圆角和控件状态应先作为 token 或共享组件样式添加。
- 避免混合视觉系统。不要将 Element Plus、Naive UI、PrimeVue 或类似的带样式组件库引入功能视图，除非其视觉效果完全隐藏在 `Wb*` 包装器之后。
- 保持工作台紧凑且 IDE 风格：活动栏、工具窗口、编辑器标签、右侧面板、细边框、紧凑行、适度圆角、无营销风格的卡片、产品表面无装饰性渐变。
- 每个 UI 基础改造应包含前端测试和生产构建。

## 批次 0：仓库与开发基线

### 目标

创建空项目骨架，建立工具链，确保仓库能够运行最小化的 Rust 服务和 Vue 前端，而不实现产品逻辑。

### 后端待办

- 为后端创建 Rust 工作区或单个 crate。
- 添加 Axum、Tokio、serde、serde_json、sqlx、reqwest、tracing、tower-http 和配置依赖项。
- 实现 `main.rs`，包含：
  - 从环境变量加载配置，
  - tracing 设置，
  - Axum 路由创建，
  - 健康检查端点。
- 添加共享错误类型及 JSON 错误响应：
  - `validation_error`、
  - `not_found`、
  - `database_error`、
  - `internal_error`。
- 添加占位应用状态对象。
- 添加 `/api/health`。

### 前端待办

- 创建 Vue 3 + Vite + TypeScript 应用。
- 添加 Pinia 和 Vue Router。
- 添加调用 `/api/health` 的最小化外壳页面。
- 添加包含以下内容的基础布局：
  - 左侧工作区侧边栏占位，
  - 主内容区域，
  - 状态/错误区域。

### 基础设施待办

- 添加 `.env.example`，包含：
  - `APP_HOST`、
  - `APP_PORT`、
  - `DATABASE_URL`、
  - `APP_DATA_DIR`、
  - `RUST_LOG`。
- 添加开发脚本：
  - 运行后端、
  - 运行前端、
  - 运行测试、
  - 运行数据库迁移。
- 记录本地 PostgreSQL 要求。

### 测试

- 后端健康检查端点测试。
- 前端冒烟测试，渲染应用外壳。
- README 中的手动运行说明。

### 人工检查点

- 仓库从干净的检出开始。
- 后端启动时不包含产品数据。
- 前端启动并显示健康状态。
- 项目结构清晰可理解。
- 尚未添加认证、向量搜索或笔记/聊天逻辑。

## 批次 1：PostgreSQL 基础与设置

### 目标

将 Rust 后端连接到 PostgreSQL，添加迁移，并实现一个供后续批次使用的简单设置存储。

### 后端待办

- 添加数据库连接池初始化。
- 添加迁移运行器。
- 创建 `settings` 表：

```text
settings
- key text primary key
- value jsonb not null
- updated_at timestamptz not null
```

- 实现设置服务：
  - 获取所有设置、
  - 获取单个设置、
  - 更新或插入设置、
  - 批量修改设置。
- 实现路由：
  - `GET /api/settings`、
  - `PATCH /api/settings`。
- 确保设置值为任意 JSON。
- 向 `/api/health` 添加数据库健康检查。

### 前端待办

- 添加 `settingsStore`。
- 添加可从外壳访问的设置窗格占位。
- 显示当前后端/数据库健康状态。
- 允许编辑原始设置 JSON 以便早期调试。

### 测试

- 针对 PostgreSQL 的迁移测试。
- 设置更新/读取测试。
- 设置补丁的 API 测试。
- 加载设置的前端测试。

### 人工检查点

- 人工可以从 UI 创建和编辑设置。
- 设置在后端重启后保持不变。
- 数据库连接失败会产生可读的错误。
- Schema 仍然小而易于检查。

## 批次 2：工作区条目模型

### 目标

使应用中每个打开的对象都成为工作区条目。笔记、聊天、智能体配置、文件、任务和设置视图后续将插入此模型。

### 数据库待办

- 创建 `workspace_items` 表：

```text
workspace_items
- id uuid primary key
- item_type text not null
- title text not null
- parent_id uuid nullable references workspace_items(id)
- sort_order int not null default 0
- metadata jsonb not null default '{}'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- 为已知初始条目类型添加约束或验证：
  - `note`、
  - `chat`、
  - `agent_config`、
  - `file`、
  - `task`、
  - `settings_view`。

### 后端待办

- 实现工作区服务：
  - 列出条目、
  - 创建条目、
  - 获取条目、
  - 更新条目、
  - 删除条目、
  - 重新排序条目、
  - 将条目移动到父级下。
- 实现路由：
  - `GET /api/workspace/items`、
  - `POST /api/workspace/items`、
  - `GET /api/workspace/items/:id`、
  - `PATCH /api/workspace/items/:id`、
  - `DELETE /api/workspace/items/:id`。
- 删除条目时不应级联到未来的领域表。在此批次中，仅存在工作区条目。

### 前端待办

- 添加 `workspaceStore`。
- 实现 `WorkspacePage`。
- 实现 `WorkspaceTree`。
- 实现路由：
  - `/workspace/:item_id`。
- 根据 `item_type` 渲染占位窗格。
- 在 UI 中支持创建/重命名/删除工作区条目。

### 测试

- 工作区 CRUD 测试。
- 父子排序测试。
- 前端测试：打开一个条目渲染对应的占位窗格。

### 人工检查点

- 人工可以创建一棵工作区树。
- 点击一个条目会更改主窗格。
- 应用现在感觉像一个工作区，而非断开的页面。
- 在此检查点，空占位符是可接受的。

## 批次 3：共享数学层

### 目标

创建笔记和聊天共同使用的前端数学渲染基础。此批次不实现笔记或聊天的持久化。

### 前端待办

- 添加 KaTeX。
- 添加共享数学配置模块：

```text
mathRenderOptions
latexDelimiters
formula error fallback behavior
copy raw latex behavior
CSS classes for inline and block math
```

- 实现 `MathRenderer.vue` 用于单个公式的只读渲染。
- 实现可检测文本中完整行内/块级公式的工具函数：
  - 行内：`$...$`、`\(...\)`、
  - 块级：`$$...$$`、`\[...\]`。
- 工具函数必须将不完整的公式显示为原始文本。
- 在测试工作区条目或开发路由下添加一个小型演示窗格。

### 行为要求

- 无效 LaTeX 不得导致渲染崩溃。
- 原始 LaTeX 必须保留以供复制/编辑。
- 不完整的流式类文本必须在分隔符闭合之前显示为原始文本。

### 测试

- 渲染有效行内公式。
- 渲染有效块级公式。
- 无效公式安全降级。
- 不完整公式保持原始文本。
- 混合普通文本和公式按顺序渲染。

### 人工检查点

- 人工可以可视化检查公式。
- 公式样式对未来笔记和聊天均可接受。
- 错误降级可见且不具破坏性。

## 批次 4：Tiptap 笔记核心

### 目标

使用 Tiptap/ProseMirror JSON 作为来源真值，将结构化笔记实现为工作区条目。

### 数据库待办

- 创建 `notes` 表：

```text
notes
- id uuid primary key
- workspace_item_id uuid references workspace_items(id)
- title text not null
- document_json jsonb not null
- plain_text text not null default ''
- format text not null default 'tiptap_json'
- metadata jsonb not null default '{}'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- 创建 `note_revisions` 表：

```text
note_revisions
- id uuid primary key
- note_id uuid references notes(id)
- document_json jsonb not null
- plain_text text not null default ''
- reason text nullable
- created_by text not null
- agent_run_id uuid nullable
- created_at timestamptz not null
```

### 后端待办

- 实现笔记服务：
  - 创建笔记并关联工作区条目、
  - 按 ID 获取笔记、
  - 按工作区条目 ID 获取笔记、
  - 更新笔记文档、
  - 更新标题、
  - 列出修订版本、
  - 恢复修订版本。
- 每次笔记更新时，为先前版本创建修订记录。
- 从文档 JSON 生成/更新 `plain_text`。
- 实现路由：
  - `GET /api/notes`、
  - `POST /api/notes`、
  - `GET /api/notes/:id`、
  - `PATCH /api/notes/:id`、
  - `DELETE /api/notes/:id`、
  - `GET /api/notes/:id/revisions`、
  - `POST /api/notes/:id/revisions/:revision_id/restore`。

### 前端待办

- 添加 Tiptap Vue 3。
- 添加 `@tiptap/starter-kit`。
- 添加 `@tiptap/extension-mathematics`。
- 添加 KaTeX CSS。
- 实现 `NoteEditorPane`。
- 实现 `TiptapNoteEditor`。
- 加载/保存 `document_json`。
- 显示保存状态。
- 显示修订版本列表。
- 从工作区 UI 添加笔记创建。

### 测试

- 创建笔记会创建对应的工作区条目。
- 保存段落内容为 Tiptap JSON。
- 纯文本投射更新。
- 更新时创建修订版本。
- 恢复修订版本可正常工作。
- 工作区路由打开笔记窗格。

### 人工检查点

- 人工可以从工作区树创建笔记。
- 人工可以输入普通文本并保存。
- 刷新应用后笔记内容保留。
- 修订版本存在且可以恢复。

## 批次 5：笔记中的即时公式编辑

### 目标

使笔记公式表现得像即时可编辑的渲染数学：输入公式并按下 Enter 后，它变成渲染的数学节点。

### 前端待办

- 为 Tiptap Mathematics 扩展配置：
  - `inlineMath`、
  - `blockMath`、
  - KaTeX 渲染选项，设置 `throwOnError: false`。
- 添加快捷输入规则或键盘处理：
  - `$E = mc^2$` + Enter → `inlineMath` 节点、
  - `$$ ... $$` + Enter → `blockMath` 节点。
- 添加公式插入命令：
  - 插入行内公式、
  - 插入块级公式。
- 添加 `FormulaEditPopover`：
  - 用户点击公式时打开、
  - 显示原始 LaTeX、
  - 更新 `attrs.latex`、
  - 重新渲染公式。
- 确保无效 LaTeX 不会破坏编辑器。

### 后端待办

- 验证笔记更新可以持久化 `inlineMath` 和 `blockMath` 节点。
- 确保 `plain_text` 投射包含可读的 LaTeX 文本。

### 测试

- 输入行内公式并按 Enter 创建 `inlineMath` JSON。
- 输入块级公式并按 Enter 创建 `blockMath` JSON。
- 点击公式允许编辑原始 LaTeX。
- 无效公式持久化并显示降级。
- 保存/重新加载保持公式节点完整。

### 人工检查点

- 人工可以输入 `$E=mc^2$`，按 Enter 并立即看到渲染的数学公式。
- 人工可以编辑现有公式而不丢失原始 LaTeX。
- 公式编辑体验足够稳定，可用于真实笔记。

## 批次 6：对话作为工作区条目

### 目标

将 LLM 对话窗口实现为像笔记一样打开的工作区条目，暂不调用真实 LLM 提供商。

### 数据库待办

- 创建 `conversations` 表：

```text
conversations
- id uuid primary key
- workspace_item_id uuid references workspace_items(id)
- title text not null
- status text not null default 'active'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- 创建 `messages` 表：

```text
messages
- id uuid primary key
- conversation_id uuid references conversations(id)
- parent_message_id uuid nullable
- role text not null
- content text not null
- content_json jsonb nullable
- created_at timestamptz not null
```

- 创建 `message_events` 表：

```text
message_events
- id uuid primary key
- conversation_id uuid references conversations(id)
- message_id uuid nullable
- agent_run_id uuid nullable
- event_type text not null
- payload jsonb not null
- created_at timestamptz not null
```

### 后端待办

- 实现对话服务：
  - 创建聊天并关联工作区条目、
  - 列出对话、
  - 获取对话、
  - 列出消息、
  - 追加消息、
  - 追加消息事件。
- 实现路由：
  - `GET /api/conversations`、
  - `POST /api/conversations`、
  - `GET /api/conversations/:id`、
  - `PATCH /api/conversations/:id`、
  - `DELETE /api/conversations/:id`、
  - `GET /api/conversations/:id/messages`、
  - `POST /api/conversations/:id/messages`、
  - `GET /api/conversations/:id/events`。
- 添加模拟助手响应模式用于本地测试。

### 前端待办

- 实现 `ChatPane`。
- 添加消息列表。
- 添加消息编辑器。
- 从工作区 UI 添加创建聊天。
- 使用共享数学渲染器渲染聊天消息。
- 存储原始消息内容。

### 测试

- 创建聊天创建工作区条目。
- 添加用户消息。
- 添加助手模拟消息。
- 刷新后消息保留。
- 工作区路由打开聊天窗格。
- 聊天渲染器渲染完整公式，不完整公式保持原始文本。

### 人工检查点

- 人工可以从与笔记相同的工作区树打开聊天窗口。
- 聊天窗口感觉像文档类对象。
- 聊天输出中的数学公式渲染与笔记保持一致。

## 批次 7：SSE 流式传输基础设施

### 目标

为聊天和未来智能体添加流式传输基础设施，在接入真实提供商之前仍使用模拟流。

### 后端待办

- 添加 SSE 路由：
  - `GET /api/events?conversation_id=...&agent_run_id=...`。
- 实现事件广播器。
- 将流式消息事件持久化到 `message_events`。
- 添加模拟流端点行为：
  - 发出 `llm.delta`、
  - 发出 `llm.done`、
  - 存储最终助手消息。

### 前端待办

- 添加 `streamStore`。
- 将 `ChatPane` 连接到 SSE。
- 将增量 token 追加到活动消息。
- 在流式传输期间安全地重新渲染数学公式。
- 流完成后，运行完整的消息渲染。
- 显示流状态和错误。

### 测试

- SSE 事件顺序测试。
- 增量持久化测试。
- 前端 token 追加测试。
- 流期间不完整公式保持原始文本。
- 流完成后公式渲染完成。

### 人工检查点

- 人工可以发送模拟消息并观看其流式传输。
- 流式传输不会破坏公式。
- 刷新后可以检查存储的事件。

## 批次 8：LLM 提供商与手动请求配置

### 目标

实现 OpenAI 兼容的提供商配置和可手动编辑的请求配置。这是与 UI 仅参数滑块的核心差异化功能。

### 数据库待办

- 创建 `llm_providers`：

```text
llm_providers
- id uuid primary key
- name text not null
- base_url text not null
- api_key text nullable
- default_model text nullable
- enabled boolean not null default true
- created_at timestamptz not null
- updated_at timestamptz not null
```

- 创建 `llm_request_profiles`：

```text
llm_request_profiles
- id uuid primary key
- name text not null
- provider_id uuid references llm_providers(id)
- endpoint_path text not null default '/chat/completions'
- method text not null default 'POST'
- base_body jsonb not null
- headers jsonb not null default '{}'
- message_injection_mode text not null default 'replace_messages'
- enabled boolean not null default true
- created_at timestamptz not null
- updated_at timestamptz not null
```

- 创建 `message_llm_calls`：

```text
message_llm_calls
- id uuid primary key
- conversation_id uuid references conversations(id)
- message_id uuid nullable references messages(id)
- agent_run_id uuid nullable
- provider_id uuid nullable
- request_profile_id uuid nullable
- request_body jsonb not null
- response_body jsonb nullable
- status text not null
- error text nullable
- started_at timestamptz not null
- finished_at timestamptz nullable
```

### 后端待办

- 实现提供商 CRUD。
- 实现请求配置 CRUD。
- 实现请求构建器，合并顺序为：
  1. 配置 `base_body`、
  2. 智能体运行时 LLM 覆盖、
  3. 当前请求 `runtime_overrides`、
  4. 当前请求 `raw_body_overrides`、
  5. 后端注入的消息。
- 首先实现 `replace_messages` 注入模式。
- 保留未知/自定义 JSON 字段。
- 在提供商调用前归档最终请求 JSON。
- 归档响应体或错误。
- 添加配置测试端点：
  - `POST /api/llm/request-profiles/:id/test`。

### 前端待办

- 添加 LLM 设置窗格。
- 添加提供商编辑器。
- 添加请求配置编辑器。
- 添加 `base_body` 的 JSON 体编辑器。
- 添加 `headers` 的 JSON 编辑器。
- 添加测试调用 UI。
- 添加最终请求 JSON 预览。

### 测试

- 提供商 CRUD 测试。
- 请求配置 CRUD 测试。
- JSON 合并顺序测试。
- 未知提供商参数被保留。
- 缺少 `messages` 时被注入。
- 已有 `messages` 在 `replace_messages` 模式下被替换。
- 最终请求被归档。

### 人工检查点

- 人工可以手写提供商请求 JSON。
- 应用显示将发送的确切 JSON。
- 未知/自定义字段不被剥离。
- 失败的提供商调用可被理解。

## 批次 9：真实 LLM 聊天集成

### 目标

通过请求配置系统将聊天窗格连接到真实的 OpenAI 兼容流式调用。

### 后端待办

- 实现 `llm_gateway`。
- 使用 reqwest 实现流式提供商请求。
- 支持 OpenAI 兼容的流式增量。
- 将提供商流转换为内部事件：
  - `llm.request.created`、
  - `llm.delta`、
  - `llm.done`、
  - `llm.error`。
- 存储最终助手消息。
- 存储最终响应体或规范化流摘要。
- 实现超时和提供商错误映射：
  - `provider_error`、
  - `provider_timeout`、
  - `provider_bad_response`、
  - `llm_request_build_failed`。

### 前端待办

- 让 `ChatPane` 可选择 LLM 请求配置。
- 让消息发送包含：
  - `llm_request_profile_id`、
  - `runtime_overrides`、
  - `raw_body_overrides`。
- 添加 `LlmRequestInspector`。
- 显示每个助手响应的最终请求 JSON。

### 测试

- 模拟提供商流式集成。
- 可手动测试真实提供商调用。
- 最终请求 JSON 匹配所选配置和覆盖。
- 提供商错误持久化到 `message_llm_calls`。
- 前端显示流和最终归档消息。

### 人工检查点

- 人工可以配置提供商并与真实模型聊天。
- 人工可以检查发送给提供商的确切 JSON。
- 聊天数学渲染对真实模型输出仍然有效。

## 批次 10：提示块

### 目标

添加可独立复用的文本块，可注入 LLM 请求，无需将用户所有内容放在一个预设中。

### 数据库待办

- 创建 `prompt_blocks`：

```text
prompt_blocks
- id uuid primary key
- name text not null
- description text nullable
- content text not null
- block_type text not null
- enabled boolean not null default true
- sort_order int not null default 0
- created_at timestamptz not null
- updated_at timestamptz not null
```

### 后端待办

- 实现提示块 CRUD。
- 实现提示注入服务。
- 在聊天发送请求中支持选定的提示块 ID。
- 默认注入顺序：
  1. 按 `sort_order` 排列的选定提示块、
  2. 对话上下文、
  3. 当前用户消息。
- 添加归档元数据，显示使用了哪些提示块。

### 前端待办

- 添加 `PromptBlocksPane` 或用于提示块管理的工作区条目窗格。
- 在 `ChatPane` 中添加提示块选择器。
- 支持多选。
- 支持预览。
- 支持每条消息的临时选择。

### 测试

- 提示块 CRUD。
- 多块注入顺序。
- 禁用的块不被注入。
- 选定的块 ID 被归档。
- 最终请求 JSON 包含预期的注入文本。

### 人工检查点

- 人工可以创建独立的可复用文本块。
- 人工可以为一条消息组合多个块。
- 最终请求 JSON 使注入透明可见。

## 批次 11：用户自定义智能体配置

### 目标

实现可配置的智能体，不含硬编码的智能体类型。

### 数据库待办

- 创建 `agent_configs`：

```text
agent_configs
- id uuid primary key
- workspace_item_id uuid nullable references workspace_items(id)
- name text not null
- description text nullable
- enabled boolean not null default true
- llm_request_profile_id uuid references llm_request_profiles(id)
- system_prompt text nullable
- selected_prompt_block_ids uuid[] not null default '{}'
- tool_permissions jsonb not null default '{}'
- runtime_config jsonb not null default '{}'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- 创建 `agent_runs`：

```text
agent_runs
- id uuid primary key
- agent_config_id uuid references agent_configs(id)
- conversation_id uuid nullable references conversations(id)
- status text not null
- input jsonb not null
- output jsonb nullable
- error text nullable
- started_at timestamptz nullable
- finished_at timestamptz nullable
- created_at timestamptz not null
```

### 后端待办

- 实现智能体配置 CRUD。
- 创建智能体配置工作区条目。
- 实现智能体运行服务：
  - 创建运行、
  - 使用智能体配置执行一个 LLM 请求、
  - 持久化运行状态、
  - 持久化事件。
- 智能体配置决定：
  - 请求配置、
  - 系统提示、
  - 选定的提示块、
  - 运行时配置、
  - 允许的工具。
- 不基于智能体名称/类型分支逻辑。

### 前端待办

- 添加 `AgentConfigPane`。
- 编辑：
  - 名称、
  - 描述、
  - 请求配置、
  - 系统提示、
  - 默认提示块、
  - 运行时配置 JSON、
  - 工具权限 JSON。
- 在 `ChatPane` 中添加智能体选择器。
- 添加智能体运行面板。

### 测试

- 智能体配置 CRUD。
- 智能体工作区条目打开配置窗格。
- 智能体运行持久化状态转换。
- 智能体使用配置的请求配置。
- 智能体使用配置的提示块。
- 不存在硬编码的智能体类型行为。

### 人工检查点

- 人工可以创建命名智能体。
- 智能体的行为明显地由其配置控制。
- 运行智能体会创建可检查的事件和运行记录。

## 批次 12：工具注册表与会话/笔记工具

### 目标

添加权限检查的工具系统，并实现用于读取对话和修改笔记的安全工具。

### 后端待办

- 实现工具注册表抽象：

```text
tool.name
tool.description
tool.input_schema
tool.output_schema
tool.permission_key
tool.execute()
```

- 实现工具：
  - `conversation.read_messages`、
  - `conversation.summarize_context`、
  - `note.list`、
  - `note.read`、
  - `note.create`、
  - `note.update_document`、
  - `note.create_revision`。
- 每次工具调用前检查 `agent_configs.tool_permissions`。
- 持久化工具调用事件：
  - `agent.tool_call`、
  - `agent.tool_result`、
  - `agent.tool_error`。
- 对于智能体进行的笔记更新，创建 `created_by = agent` 的修订记录。

### 前端待办

- 添加工具权限编辑器。
- 添加显示工具调用的智能体运行事件时间线。
- 添加智能体修改笔记时的视觉指示。

### 测试

- 工具权限允许路径。
- 工具权限拒绝路径。
- 智能体笔记更新创建修订记录。
- 工具调用事件持久化。
- 工具错误不会使整个应用崩溃。

### 人工检查点

- 人工可以看到智能体被允许使用哪些工具。
- 人工可以在智能体运行后检查工具调用。
- 智能体创建的笔记更改可通过修订记录回滚。

## 批次 13：文件与附件

### 目标

添加文件源、文档元数据、对话附件和文件读取工具。此批次不实现向量搜索。

### 数据库待办

- 创建 `file_sources`：

```text
file_sources
- id uuid primary key
- workspace_item_id uuid nullable references workspace_items(id)
- name text not null
- source_type text not null
- original_path text nullable
- storage_policy text not null
- enabled boolean not null default true
- metadata jsonb not null default '{}'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- 创建 `documents`：

```text
documents
- id uuid primary key
- source_id uuid references file_sources(id)
- title text not null
- mime_type text nullable
- original_path text nullable
- stored_path text nullable
- hash text nullable
- parse_status text not null default 'pending'
- parse_error text nullable
- extracted_text text nullable
- metadata jsonb not null default '{}'
- created_at timestamptz not null
- updated_at timestamptz not null
```

- 创建 `attachments`：

```text
attachments
- id uuid primary key
- conversation_id uuid nullable references conversations(id)
- message_id uuid nullable references messages(id)
- filename text not null
- mime_type text nullable
- original_path text nullable
- stored_path text nullable
- hash text nullable
- metadata jsonb not null default '{}'
- created_at timestamptz not null
```

### 后端待办

- 添加文件源 CRUD。
- 支持 `reference` 和 `copy` 存储策略。
- 将文本类文件解析到 `documents.extracted_text`。
- 保存不受支持的文件，设置 `parse_status = unsupported`。
- 实现工具：
  - `file.list`、
  - `file.read`。
- 在聊天消息发送请求中支持附件。

### 前端待办

- 添加文件窗格。
- 添加文件源创建 UI。
- 显示解析状态。
- 显示提取文本预览。
- 将文件附加到聊天消息。

### 测试

- 引用文件源。
- 复制文件源。
- 文本文件解析。
- 不受支持文件状态。
- 附件随消息保存。
- 智能体文件读取遵循权限。

### 人工检查点

- 人工可以添加本地文件或文件夹源。
- 人工可以检查提取的文本。
- 聊天可以携带附件。
- 智能体仅在获得权限时才能读取文件。

## 批次 14：调度器

### 目标

添加 Rust 进程绑定的计划任务。任务仅在后台进程存活时运行；前端仅显示和触发它们。

### 数据库待办

- 创建 `scheduled_tasks`：

```text
scheduled_tasks
- id uuid primary key
- workspace_item_id uuid nullable references workspace_items(id)
- name text not null
- enabled boolean not null default true
- task_type text not null
- agent_config_id uuid nullable references agent_configs(id)
- schedule_config jsonb not null
- payload jsonb not null default '{}'
- last_run_at timestamptz nullable
- next_run_at timestamptz nullable
- created_at timestamptz not null
- updated_at timestamptz not null
```

- 创建 `task_runs`：

```text
task_runs
- id uuid primary key
- task_id uuid references scheduled_tasks(id)
- agent_run_id uuid nullable references agent_runs(id)
- status text not null
- output jsonb nullable
- error text nullable
- started_at timestamptz not null
- finished_at timestamptz nullable
```

### 后端待办

- 实现调度器服务。
- 启动时加载已启用的任务。
- 支持间隔调度：

```json
{ "kind": "interval", "seconds": 86400 }
```

- 如果引入依赖，可选支持 cron。
- 实现任务 CRUD。
- 实现手动任务运行。
- 任务类型 `agent_run` 调用配置的智能体。
- 持久化任务运行状态和错误。

### 前端待办

- 添加 `TaskPane`。
- 显示任务列表。
- 使用 JSON 调度配置创建/编辑任务。
- 将任务绑定到智能体配置。
- 手动运行按钮。
- 显示最近的任务运行。

### 测试

- 任务 CRUD。
- 手动任务运行。
- 间隔下次运行计算。
- 失败的智能体任务记录错误。
- 调度器不运行禁用的任务。

### 人工检查点

- 人工可以创建计划任务。
- 人工可以手动运行它。
- 人工可以关闭前端而后端继续运行任务。
- 停止后端则停止调度。

## 批次 15：文件写入计划、审计与回滚

### 目标

允许智能体通过可审计、可逆的管道提议和应用文件写入。

### 数据库待办

- 创建 `file_write_audits`：

```text
file_write_audits
- id uuid primary key
- agent_run_id uuid nullable references agent_runs(id)
- path text not null
- operation text not null
- before_hash text nullable
- after_hash text nullable
- before_snapshot_path text nullable
- after_snapshot_path text nullable
- diff text nullable
- created_at timestamptz not null
```

### 后端待办

- 实现写入计划 API：
  - 提议的操作、
  - 目标路径、
  - 前后内容、
  - 统一 diff。
- 实现应用写入：
  - 读取现有文件、
  - 计算前哈希、
  - 保存前快照、
  - 写入新内容、
  - 计算后哈希、
  - 保存审计记录。
- 实现回滚：
  - 从快照恢复、
  - 回滚创建操作时删除已创建的文件、
  - 回滚删除操作时恢复已删除的文件。
- 实现工具：
  - `file.preview_write`、
  - `file.apply_write`、
  - `file.rollback_write`。

### 前端待办

- 添加文件写入计划预览。
- 应用前显示 diff。
- 显示审计列表。
- 添加回滚按钮。
- 显示导致写入的智能体运行。

### 测试

- 预览写入不产生文件变更。
- 应用更新写入文件并审计。
- 应用创建写入文件并审计。
- 回滚更新恢复内容。
- 回滚创建删除文件。
- 权限拒绝阻止工具执行。

### 人工检查点

- 人工可以在应用前检查提议的文件写入。
- 已应用的写入可审计。
- 回滚在真实文件上生效。
- 在开放更广泛的智能体写入权限之前，此批次应接受额外审阅。

## 批次 16：打磨、一致性与端到端验收

### 目标

使整个产品足够连贯，可用于日常实验性使用。

### 后端待办

- 规范化所有路由的错误代码。
- 添加请求日志，敏感字段脱敏。
- 添加一致的时间戳。
- 在需要处添加分页：
  - 工作区条目、
  - 消息、
  - 事件、
  - 运行记录、
  - 审计记录。
- 为以下内容添加数据库索引：
  - 工作区父子/排序、
  - 对话消息、
  - 消息事件、
  - 智能体运行、
  - 任务运行。
- 在可行的范围内添加主要用户数据的备份/导出端点。

### 前端待办

- 改进工作区导航。
- 在所有位置添加加载和错误状态。
- 添加空白状态。
- 为未保存的笔记/配置 JSON 添加脏状态警告。
- 打磨最终请求 JSON 检查器。
- 添加聊天/智能体/任务的事件时间线。
- 使笔记和聊天中的数学样式保持一致。

### 测试

- 完整的端到端快乐路径：
  1. 创建笔记、
  2. 输入公式、
  3. 创建聊天、
  4. 调用 LLM、
  5. 检查请求 JSON、
  6. 创建提示块、
  7. 创建智能体、
  8. 智能体读取对话、
  9. 智能体修改笔记、
  10. 恢复修订版本。
- 错误路径测试：
  - 错误的提供商密钥、
  - 无效的 JSON 配置、
  - 提供商超时、
  - 无效公式、
  - 工具权限被拒绝、
  - 文件写入失败。

### 人工检查点

- 应用可用于真实的学习会话。
- 工作区模型感觉连贯。
- 笔记和聊天都能合理渲染公式。
- LLM 请求透明度有效。
- 智能体动作可检查。
- 项目已准备好进行第二轮规划。

## 推迟范围

以下内容有意不包含在上述批次中。

### 访问控制

- 管理员密码。
- API token。
- 本地免认证 / 远程认证分离。
- 多用户账户。
- 权限隔离。

### 向量检索

- pgvector。
- 嵌入提供商。
- 文档分块。
- 混合搜索。
- RAG 上下文构建器。
- 文件夹索引任务。

### 全文搜索

- PostgreSQL 全文搜索。
- 中文分词。
- 文件名/路径/正文排序。
- 对话搜索。

### 远程部署安全

- 反向代理认证。
- VPN。
- IP 白名单。
- Cloudflare Access。
- 内置认证。

### 高级自动化文件写入策略

- 是否每次写入都需要确认。
- 哪些路径被禁止。
- 是否允许删除。
- 是否允许批量修改。
- 智能体是否可以自动应用写入。