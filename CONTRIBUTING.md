# 贡献指南

感谢您对Zero编程语言项目的关注！

## 开发环境

- Rust 1.70 或更高版本
- Cargo (Rust包管理器)

## 项目结构

```
Zero-compiler/
├── src/
│   ├── main.rs          # 主程序入口
│   ├── lexer/           # 词法分析器
│   │   ├── mod.rs       # Lexer实现
│   │   └── token.rs     # Token定义
│   ├── parser/          # 语法分析器
│   │   └── mod.rs       # Parser实现
│   ├── ast/             # 抽象语法树
│   │   └── mod.rs       # AST节点定义
│   └── interpreter/     # 解释器
│       └── mod.rs       # 解释器实现
├── examples/            # 示例程序
└── tests/               # 测试用例
```

## 构建和测试

```bash
# 构建项目
cargo build

# 运行测试
cargo test

# 运行示例
cargo run examples/hello.zero
```

## 编码规范

- 遵循Rust官方编码规范
- 所有公共API都应有文档注释
- 提交前运行 `cargo fmt` 和 `cargo clippy`

## 提交流程

1. Fork本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建Pull Request

## 报告问题

如果您发现bug或有功能建议，请创建Issue并提供：

- 问题的详细描述
- 重现步骤
- 预期行为和实际行为
- 您的环境信息（操作系统、Rust版本等）

## 许可证

通过贡献代码，您同意您的贡献将在MIT许可证下发布。