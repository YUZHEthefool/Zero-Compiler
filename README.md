# Zero 编程语言

Zero是一门使用Rust实现的现代编程语言。

## 特性

- **静态类型系统**：编译期类型检查
- **简洁语法**：易于学习和使用
- **变量声明**：支持 `let` 和 `var` 关键字
- **基本数据类型**：整数、浮点数、字符串、布尔值
- **控制流**：if/else、while、for循环
- **函数定义**：支持函数声明和调用
- **运算符**：算术、比较、逻辑运算符

## 语法示例

```zero
// 变量声明
let x = 42;
var y = 3.14;
let name = "Zero";
let flag = true;

// 函数定义
fn add(a, b) {
    return a + b;
}

// 控制流
if x > 10 {
    print("x is greater than 10");
} else {
    print("x is less than or equal to 10");
}

// 循环
while x > 0 {
    x = x - 1;
}

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
│   ├── parser/          # 语法分析器
│   ├── ast/             # 抽象语法树
│   ├── semantic/        # 语义分析
│   └── interpreter/     # 解释器
├── examples/            # 示例程序
└── tests/               # 测试用例
```

## 构建和运行

```bash
# 构建项目
cargo build

# 运行编译器
cargo run -- <source_file.zero>

# 运行测试
cargo test
```

## 开发状态

🚧 项目正在开发中

## 许可证

MIT License