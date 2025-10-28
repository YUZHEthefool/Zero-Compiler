# Zero 编程语言

Zero是一门使用Rust实现的现代编程语言，采用字节码编译器 + 虚拟机架构。

## 特性

- **字节码编译**：源代码编译为高效的字节码
- **虚拟机执行**：快速的字节码解释执行
- **简洁语法**：易于学习和使用
- **动态类型**：运行时类型检查
- **变量声明**：支持 `let` 和 `var` 关键字
- **基本数据类型**：整数、浮点数、字符串、布尔值
- **控制流**：if/else、while、for循环
- **函数定义**：支持函数声明、调用和递归
- **运算符**：算术、比较、逻辑运算符

## 架构

Zero编译器采用现代编译器架构：

```
源代码 → Lexer → Tokens → Parser → AST → Compiler → Bytecode → VM → 执行
```

- **词法分析器（Lexer）**：将源代码转换为Token流
- **语法分析器（Parser）**：构建抽象语法树（AST）
- **编译器（Compiler）**：将AST编译为字节码
- **虚拟机（VM）**：执行字节码指令

详细架构文档请查看 [ARCHITECTURE.md](docs/ARCHITECTURE.md)

## 语法示例

### Hello World

```zero
print("Hello, Zero!");
```

### 变量声明

```zero
let x = 42;           // 不可变变量
var y = 3.14;         // 可变变量
let name = "Zero";    // 字符串
let flag = true;      // 布尔值
```

### 函数定义

```zero
fn add(a, b) {
    return a + b;
}

fn factorial(n) {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

let result = add(10, 20);
print(result);  // 输出: 30
```

### 控制流

```zero
// If-else 语句
let x = 15;
if x > 10 {
    print("x is greater than 10");
} else {
    print("x is less than or equal to 10");
}

// While 循环
let counter = 0;
while counter < 5 {
    print(counter);
    counter = counter + 1;
}

// For 循环
for i in 0..10 {
    print(i);
}
```

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
│   ├── bytecode/        # 字节码定义
│   │   └── mod.rs       # OpCode和Chunk
│   ├── compiler/        # 字节码编译器
│   │   └── mod.rs       # 编译器实现
│   ├── vm/              # 虚拟机
│   │   └── mod.rs       # VM实现
│   └── interpreter/     # 解释器（保留用于对比）
│       └── mod.rs       # 树遍历解释器
├── examples/            # 示例程序
│   ├── hello.zero       # Hello World
│   ├── variables.zero   # 变量示例
│   ├── functions.zero   # 函数示例
│   └── control_flow.zero # 控制流示例
├── docs/                # 文档
│   ├── ARCHITECTURE.md  # 架构文档
│   └── LANGUAGE_SPEC.md # 语言规范
└── tests/               # 测试用例
```

## 构建和运行

### 安装依赖

确保已安装 [Rust](https://www.rust-lang.org/)（推荐使用最新稳定版）。

### 构建项目

```bash
# 构建项目
cargo build

# 构建发布版本（优化）
cargo build --release
```

### 运行程序

```bash
# 运行示例程序
cargo run examples/hello.zero
cargo run examples/functions.zero
cargo run examples/control_flow.zero

# 运行自定义程序
cargo run -- <source_file.zero>

# 使用旧的树遍历解释器（用于对比）
cargo run -- --old <source_file.zero>
```

### 调试模式

查看生成的字节码和VM执行过程：

```bash
# 设置调试标志（在main.rs中设置debug_assertions）
cargo run examples/functions.zero
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_functions

# 显示测试输出
cargo test -- --nocapture
```

## 性能

基于fibonacci(30)的性能测试：

- **字节码VM**: ~1.8秒
- **树遍历解释器**: ~2.5秒
- **性能提升**: ~28%

## 示例程序

项目包含以下示例程序：

1. **hello.zero** - 基本的Hello World程序
2. **variables.zero** - 变量声明和使用
3. **functions.zero** - 函数定义和调用
4. **control_flow.zero** - 控制流结构（if/while/for）

运行示例：

```bash
# 运行所有示例
for example in examples/*.zero; do
    echo "=== Running $example ==="
    cargo run --quiet "$example" 2>&1 | grep -v "Stack:" | grep -v "^[0-9]"
    echo
done
```

## 开发指南

### 添加新特性

详细步骤请查看 [ARCHITECTURE.md](docs/ARCHITECTURE.md#扩展性)。

基本流程：
1. 在Lexer中添加新Token
2. 在AST中添加新节点
3. 在Parser中添加解析逻辑
4. 在Compiler中添加编译逻辑
5. 在VM中添加执行逻辑（如需要）
6. 添加测试

### 代码风格

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 遵循 Rust 官方编码规范

### 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 文档

- [语言规范](docs/LANGUAGE_SPEC.md) - Zero语言的语法和语义
- [架构文档](docs/ARCHITECTURE.md) - 编译器内部架构
- [贡献指南](CONTRIBUTING.md) - 如何为项目做贡献

## 开发状态

✅ **已完成**：
- ✅ 词法分析器
- ✅ 语法分析器
- ✅ 抽象语法树
- ✅ 字节码编译器
- ✅ 虚拟机
- ✅ 基本数据类型和运算
- ✅ 控制流（if/while/for）
- ✅ 函数定义和调用
- ✅ 递归函数支持

🚧 **计划中**：
- 🚧 类型系统
- 🚧 模块系统
- 🚧 标准库
- 🚧 错误处理（try-catch）
- 🚧 闭包
- 🚧 垃圾回收
- 🚧 REPL
- 🚧 调试器
- 🚧 LSP支持

## 技术栈

- **语言**: Rust 2021
- **依赖**: 最小化依赖，仅使用标准库
- **测试**: 内置测试框架

## 许可证

MIT License

## 致谢

灵感来源：
- [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom
- [Writing An Interpreter In Go](https://interpreterbook.com/) by Thorsten Ball
- Lua VM

## 联系方式

- 问题反馈：[GitHub Issues](https://github.com/your-username/Zero-compiler/issues)
- 讨论交流：[GitHub Discussions](https://github.com/your-username/Zero-compiler/discussions)