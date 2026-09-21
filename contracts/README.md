# contracts/ — 跨语言契约

**契约是唯一真相源。** 所有语言（Dart / Java / Go / Rust / Python / TypeScript）的接口与 SDK 均从本目录生成，禁止手写接口定义。

## 目录规划

| 目录 | 内容 | 生成物 |
|------|------|--------|
| `openapi/` | 对外 REST 接口（OpenAPI 3.0） | Dart / Java / Go / TypeScript SDK |
| `proto/` | 内部服务间通信（gRPC + Protobuf） | 各语言 stub |
| `asyncapi/` | Kafka 事件 schema（AsyncAPI） | 事件生产 / 消费代码 |
| `schemas/` | 通用 JSON Schema（播放事件、用户操作） | 校验代码 / 类型定义 |

## 约定

1. **契约先行**：任何接口变更先改契约，再改实现。
2. **禁止手写**：实现侧只消费生成物，避免接口定义漂移。
3. **兼容性校验**：CI 对契约做向后兼容性检查，破坏性变更必须升版本。
4. **版本化**：REST 走路径版本（`/v1`），Protobuf 走 package 版本，事件 schema 带版本字段。
5. **上游知识沉淀**：塞壬唱片 API 的端点与字段知识见 [`docs/monster-siren-api.md`](../docs/monster-siren-api.md)。
6. **fixture 验证**：`tools/siren-ref` 可拉取真实响应生成 JSON fixture，用于契约测试。

## 任务清单

- [ ] 初始化 OpenAPI 规范（用户、账号、收藏、评论、帖子、播放事件）
- [ ] 初始化 Protobuf 服务定义（Java ↔ Go）
- [ ] 初始化 AsyncAPI 事件定义（播放事件、点赞、评论、通知）
- [ ] 接入代码生成流水线（openapi-generator / protoc / asyncapi-codegen）
- [ ] CI 兼容性校验

## 状态

规划中，尚未创建具体契约文件。
