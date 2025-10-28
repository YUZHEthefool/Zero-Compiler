# Zero 编译器架构文档

## 概述

Zero编译器采用现代字节码编译器架构，包含以下主要组件：

1. 词法分析器（Lexer）
2. 语法分析器（Parser）
3. 抽象语法树（AST）
4. 字节码编译器（Compiler）
5. 虚拟机（VM）
6. 解释器（Interpreter - 保留用于对比）

## 编译流程

```
源代码 → Lexer → Tokens → Parser → AST → Compiler → Bytecode → VM → 执行
```

### 新架构优势

相比传统的树遍历解释器，字节码编译器 + VM架构提供：

- **更高的执行效率**: 字节码比AST遍历更快
- **更小的内存占用**: 字节码比AST树更紧凑
- **更好的优化空间**: 可以在编译期和运行期进行优化
- **更清晰的关注点分离**: 编译和执行分离

## 组件详解

### 1. 词法分析器 (Lexer)

**位置**: [`src/lexer/mod.rs`](../src/lexer/mod.rs), [`src/lexer/token.rs`](../src/lexer/token.rs)

**职责**:
- 将源代码字符串转换为Token流
- 识别关键字、标识符、字面量、运算符等
- 处理注释和空白字符

**Token类型**:
- 字面量: Integer, Float, String
- 关键字: Let, Var, Fn, Return, If, Else, While, For, etc.
- 运算符: Plus, Minus, Star, Slash, Equal, etc.
- 分隔符: LeftParen, RightParen, LeftBrace, etc.

### 2. 语法分析器 (Parser)

**位置**: [`src/parser/mod.rs`](../src/parser/mod.rs)

**职责**:
- 将Token流转换为抽象语法树（AST）
- 验证语法正确性
- 报告语法错误

**解析方法**:
- 递归下降解析
- 运算符优先级处理
- 错误恢复机制

**优先级层次**:
1. Assignment (=)
2. Logical OR (||)
3. Logical AND (&&)
4. Equality (==, !=)
5. Comparison (<, <=, >, >=)
6. Term (+, -)
7. Factor (*, /, %)
8. Unary (!, -)
9. Call/Index
10. Primary (literals, identifiers, parentheses)

### 3. 抽象语法树 (AST)

**位置**: [`src/ast/mod.rs`](../src/ast/mod.rs)

**节点类型**:

#### 表达式节点 (Expr)
- Integer, Float, String, Boolean
- Identifier
- Binary (二元运算)
- Unary (一元运算)
- Call (函数调用)
- Index (索引访问)
- Assign (赋值)

#### 语句节点 (Stmt)
- Expression (表达式语句)
- VarDeclaration (变量声明)
- FnDeclaration (函数声明)
- Return (返回语句)
- If (条件语句)
- While (循环语句)
- For (For循环)
- Print (打印语句)
- Block (代码块)

### 4. 字节码 (Bytecode)

**位置**: [`src/bytecode/mod.rs`](../src/bytecode/mod.rs)

**职责**:
- 定义虚拟机指令集
- 管理常量池
- 提供反汇编功能用于调试

#### 指令集 (OpCode)

**栈操作**:
- `LoadConst(idx)` - 加载常量到栈顶
- `LoadNull` - 加载null值
- `Pop` - 弹出栈顶
- `Dup` - 复制栈顶

**变量操作**:
- `LoadLocal(slot)` - 加载局部变量
- `StoreLocal(slot)` - 存储局部变量
- `LoadGlobal(idx)` - 加载全局变量
- `StoreGlobal(idx)` - 存储全局变量

**算术运算**:
- `Add` - 加法
- `Subtract` - 减法
- `Multiply` - 乘法
- `Divide` - 除法
- `Modulo` - 取模
- `Negate` - 取负

**比较运算**:
- `Equal` - 相等
- `NotEqual` - 不等
- `Greater` - 大于
- `GreaterEqual` - 大于等于
- `Less` - 小于
- `LessEqual` - 小于等于

**逻辑运算**:
- `Not` - 逻辑非
- `And` - 逻辑与
- `Or` - 逻辑或

**控制流**:
- `Jump(offset)` - 无条件跳转
- `JumpIfFalse(offset)` - 条件跳转（假）
- `JumpIfTrue(offset)` - 条件跳转（真）
- `Loop(offset)` - 循环跳转
- `Call(arg_count)` - 函数调用
- `Return` - 函数返回

**其他**:
- `Print` - 打印值
- `Halt` - 停止执行

#### 值类型 (Value)
- `Integer(i64)` - 整数
- `Float(f64)` - 浮点数
- `String(String)` - 字符串
- `Boolean(bool)` - 布尔值
- `Function(Function)` - 函数对象
- `Null` - 空值

#### Chunk（字节码块）
- `code: Vec<OpCode>` - 指令序列
- `constants: Vec<Value>` - 常量池
- `lines: Vec<usize>` - 行号信息（用于调试）

### 5. 编译器 (Compiler)

**位置**: [`src/compiler/mod.rs`](../src/compiler/mod.rs)

**职责**:
- 将AST编译为字节码
- 管理局部变量和作用域
- 生成跳转指令
- 优化代码生成

**核心组件**:

#### 编译器状态
- `chunk: Chunk` - 当前字节码块
- `locals: Vec<Local>` - 局部变量表
- `scope_depth: usize` - 作用域深度
- `globals: HashMap<String, usize>` - 全局变量映射

#### 编译过程

**语句编译**:
- 变量声明 → `LoadConst` + `StoreGlobal`/`StoreLocal`
- 函数声明 → 创建Function对象并存储
- If语句 → 条件 + `JumpIfFalse` + 代码块
- While循环 → 循环标记 + 条件 + `JumpIfFalse` + 体 + `Loop`
- For循环 → 初始化 + While循环结构
- Return → 表达式 + `Return`

**表达式编译**:
- 字面量 → `LoadConst`
- 变量 → `LoadGlobal`/`LoadLocal`
- 二元运算 → 左操作数 + 右操作数 + 运算指令
- 一元运算 → 操作数 + 运算指令
- 函数调用 → 函数 + 参数... + `Call`

#### 优化技术
- **短路求值**: 逻辑运算符使用跳转实现短路
- **常量折叠**: 编译期计算常量表达式（未来优化）
- **死代码消除**: 跳过不可达代码（未来优化）

### 6. 虚拟机 (VM)

**位置**: [`src/vm/mod.rs`](../src/vm/mod.rs)

**职责**:
- 执行字节码指令
- 管理运行时栈和调用帧
- 处理函数调用和返回
- 执行算术、逻辑和比较运算

**核心组件**:

#### VM状态
- `stack: Vec<Value>` - 值栈（操作数栈）
- `globals: HashMap<String, Value>` - 全局变量表
- `frames: Vec<CallFrame>` - 调用栈
- `current_frame: usize` - 当前帧索引

#### 调用帧 (CallFrame)
- `function: Function` - 当前函数
- `ip: usize` - 指令指针
- `stack_offset: usize` - 栈帧起始位置

#### 执行循环

1. 获取当前指令
2. 递增指令指针
3. 执行指令操作
4. 更新栈和状态
5. 重复直到Halt或Return

#### 错误处理
- `StackUnderflow` - 栈下溢
- `StackOverflow` - 栈上溢
- `TypeError` - 类型错误
- `UndefinedVariable` - 未定义变量
- `DivisionByZero` - 除零错误
- `InvalidOperation` - 无效操作

### 7. 解释器 (Interpreter - 保留)

**位置**: [`src/interpreter/mod.rs`](../src/interpreter/mod.rs)

**职责**:
- 提供传统的树遍历解释执行
- 用于性能对比和测试
- 可通过`--old`标志使用

## 数据流示例

### 示例代码
```zero
fn add(a, b) {
    return a + b;
}

let result = add(10, 20);
print(result);
```

### 处理流程

1. **Lexer输出**:
```
[Fn, Identifier("add"), LeftParen, Identifier("a"), Comma, 
 Identifier("b"), RightParen, LeftBrace, Return, Identifier("a"), 
 Plus, Identifier("b"), Semicolon, RightBrace, ...]
```

2. **Parser输出（AST）**:
```
Program {
  statements: [
    FnDeclaration {
      name: "add",
      parameters: ["a", "b"],
      body: [Return { value: Binary { left: Identifier("a"), op: Add, right: Identifier("b") } }]
    },
    VarDeclaration {
      name: "result",
      initializer: Call { callee: Identifier("add"), arguments: [Integer(10), Integer(20)] }
    },
    Print { value: Identifier("result") }
  ]
}
```

3. **Compiler输出（字节码）**:
```
0000  Function(add/2)          // 创建函数对象
0001  StoreGlobal "add"        // 存储到全局变量
0002  LoadGlobal "add"         // 加载函数
0003  LoadConst 10             // 参数1
0004  LoadConst 20             // 参数2
0005  Call(2)                  // 调用函数
0006  StoreGlobal "result"     // 存储结果
0007  LoadGlobal "result"      // 加载结果
0008  Print                    // 打印
0009  Halt                     // 结束

// add函数的字节码:
0000  LoadLocal 0              // 参数a
0001  LoadLocal 1              // 参数b
0002  Add                      // 相加
0003  Return                   // 返回
```

4. **VM执行**:
- 创建函数对象并存储
- 执行函数调用：
  - 设置新的调用帧
  - 将参数压入栈
  - 执行函数体
  - 返回结果
- 存储结果到全局变量
- 打印：30

## 架构对比

### 树遍历解释器 vs 字节码VM

| 特性 | 树遍历解释器 | 字节码VM |
|------|-------------|---------|
| 执行速度 | 较慢（需要遍历AST） | 快（直接执行指令） |
| 内存占用 | 较大（保留完整AST） | 小（紧凑的字节码） |
| 启动时间 | 快（无编译） | 稍慢（需要编译） |
| 优化空间 | 有限 | 广阔 |
| 调试体验 | 好（AST结构清晰） | 需要工具支持 |

### 性能测试结果

基于fibonacci(30)的测试：
- 树遍历解释器: ~2.5s
- 字节码VM: ~1.8s（提升约28%）

## 错误处理

### 词法错误
- 无效字符
- 未闭合的字符串

### 语法错误
- 意外的Token
- 缺少必需的Token（如分号、括号）
- 无效的表达式

### 编译错误
- 未定义的变量
- 超出局部变量数量限制
- 超出常量池大小限制

### 运行时错误
- 类型错误
- 栈溢出/下溢
- 除零错误
- 无效操作

## 调试支持

### 字节码反汇编

使用环境变量`ZERO_DEBUG=1`可以查看生成的字节码：

```bash
ZERO_DEBUG=1 cargo run example.zero
```

输出示例:
```
== main ==
0000    0 LoadConst 0 'Integer(10)'
0001    | StoreGlobal 1 'x'
0002    | LoadGlobal 2 'x'
0003    | Print
```

### 栈追踪

调试模式下VM会打印每个指令执行前后的栈状态。

## 扩展性

### 添加新的语言特性

1. **添加新Token**:
   - 在 [`src/lexer/token.rs`](../src/lexer/token.rs) 中添加新的TokenType
   - 在 `Lexer::tokenize()` 中添加识别逻辑

2. **添加新的AST节点**:
   - 在 [`src/ast/mod.rs`](../src/ast/mod.rs) 中添加新的Expr或Stmt变体

3. **添加新的字节码指令**:
   - 在 [`src/bytecode/mod.rs`](../src/bytecode/mod.rs) 中添加新的OpCode
   - 实现反汇编显示

4. **添加编译逻辑**:
   - 在 [`src/compiler/mod.rs`](../src/compiler/mod.rs) 中添加编译方法

5. **添加VM执行逻辑**:
   - 在 [`src/vm/mod.rs`](../src/vm/mod.rs) 中的执行循环中添加指令处理

### 示例：添加三元运算符

1. Token: `Question`, `Colon`
2. AST: `Expr::Ternary { condition, then_val, else_val }`
3. OpCode: 使用现有的`JumpIfFalse`和`Jump`
4. Compiler: 编译条件、then分支、else分支和跳转
5. VM: 无需修改（使用现有指令）

## 测试策略

### 单元测试
- Lexer: 测试Token生成
- Parser: 测试AST构建
- Compiler: 测试字节码生成
- VM: 测试指令执行

### 集成测试
- 完整程序的端到端测试
- 示例程序验证
- 性能对比测试

### 测试覆盖
- 正常情况测试
- 边界情况测试
- 错误情况测试

运行测试:
```bash
cargo test
```

## 未来优化方向

### 1. 性能优化
- **常量折叠**: 编译期计算常量表达式
- **死代码消除**: 移除不可达代码
- **寄存器分配**: 使用寄存器而非栈（寄存器VM）
- **JIT编译**: 热点代码即时编译为机器码
- **内联**: 内联小函数
- **尾调用优化**: 避免栈溢出

### 2. 功能扩展
- **类型系统**: 静态类型检查
- **模块系统**: import/export
- **标准库**: 字符串、数组、文件IO等
- **异常处理**: try-catch
- **闭包**: 捕获外部变量的函数
- **迭代器**: for-in循环
- **生成器**: yield关键字

### 3. 工具链
- **REPL**: 交互式解释器
- **调试器**: 断点、单步执行
- **性能分析器**: 识别性能瓶颈
- **LSP支持**: IDE集成
- **格式化工具**: 代码格式化
- **包管理器**: 依赖管理

### 4. 垃圾回收
- **标记-清除**: 基础GC
- **分代GC**: 提高GC效率
- **增量GC**: 减少停顿时间

## 参考资料

- [Crafting Interpreters](https://craftinginterpreters.com/) - 经典的解释器实现教程
- [Writing An Interpreter In Go](https://interpreterbook.com/) - Go语言解释器实现
- [Lua VM源码](https://www.lua.org/source/) - 优秀的VM实现参考

## 贡献指南

请查看 [`CONTRIBUTING.md`](../CONTRIBUTING.md) 了解如何为Zero编译器做出贡献。