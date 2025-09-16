# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 常用开发命令

- `pnpm dev` - 启动开发服务器（前端）
- `pnpm tauri:dev` - 启动完整的 Tauri 开发环境（包含前端和 Rust 后端）
- `pnpm build` - 构建前端项目（包括 TypeScript 类型检查）
- `pnpm tauri:build` - 构建完整的 Tauri 应用程序
- `pnpm type-check` - 运行 TypeScript 类型检查
- `pnpm preview` - 预览构建后的前端应用

## 项目架构

这是一个基于 Tauri v2 的桌面应用程序，用作仓库工作空间管理器（Repository Workspace Manager）。

### 技术栈
- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Tauri
- **状态管理**: Pinia
- **样式**: Tailwind CSS v4
- **UI组件**: shadcn-vue 兼容的组件库

### 项目结构
- `.kiro` - Kiro IDE配置文件
  - `specs` 文档存放文件夹
- `src/` - Vue 前端源码
  - `components/ui/` - UI 组件库
  - `stores/` - Pinia 状态管理
  - `lib/` - 工具函数
- `src-tauri/` - Rust 后端源码
  - `src/lib.rs` - 主要的 Tauri 应用逻辑
  - `src/main.rs` - 应用入口点

### 前后端通信
- 使用 Tauri 的 `invoke` API 进行前后端通信
- Rust 命令通过 `#[tauri::command]` 宏导出
- 当前实现了 `greet` 命令作为示例

### 开发配置
- Vite 开发服务器运行在端口 1420
- 使用 `@/` 作为 `src/` 目录的路径别名
- TypeScript 配置包含严格的类型检查
- Tailwind CSS v4 通过 Vite 插件集成

### 包管理器
项目使用 pnpm 作为包管理器，配置文件为 `pnpm-lock.yaml`。

### 开发规范
- 当开始执行一个任务时，首先修改任务所属区块的状态，当其开始时，将其修改为进行中[-], 当任务完成时，将其标记为已完成[x].
- 采用TDD的模式进行开发，采用RGR的方式进行。