开发 files-explorer（Tauri2+Rust 文件管理器），严格遵守项目重构后开发约束，生成代码必须满足以下全部规则，违规代码不予采用：
### Rust后端约束
1. 所有 #[cfg(target_os)] 仅允许放在 src-tauri/src/platform/，commands/fs/search/preview/core 禁止任何平台判断；
2. 系统目录统一调用 crate::platform::paths()，禁止硬编码 %APPDATA / ~/Library / ~/.config、手动拼接 / \；
3. 文件读写、遍历、元数据、快捷方式解析全部使用 core::fs_helper，禁止裸调用 std::fs；
4. 所有函数、tauri command 返回 Result<T, AppError>，禁止无兜底 unwrap，错误自动记录 tracing 日志；
5. 新增系统托盘、快捷键、通知、窗口行为等原生能力，扩展 platform 下Trait，业务层只调用抽象接口；
6. Windows/mac/linux专属依赖使用 Cargo 条件依赖，不全局引入；
7. 不修改原有文件遍历、搜索、预览业务算法，仅替换底层调用接口，原有功能逻辑完全保留。

### 前端约束
1. 快捷键统一使用 utils/platform.ts 的 META_KEY，禁止硬编码 Ctrl/Cmd；
2. 系统目录、日志目录通过 invoke 获取，前端不处理路径兼容；
3. 所有后端请求使用封装后的统一invoke，标准化处理AppError错误弹窗。

### 输出要求
生成代码前标注改动目录，仅在允许修改范围新增/替换代码；业务逻辑原有循环、匹配、过滤规则一行不改动；若需求涉及平台差异，全部收敛至platform适配层实现。
