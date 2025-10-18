# Zero 编译器架构文档

## 概述

Zero编译器采用传统的编译器架构，包含以下主要组件：

1. 词法分析器（Lexer）
2. 语法分析器（Parser）
3. 抽象语法树（AST）
4. 解释器（Interpreter）

## 编译流程

```
源代码 → Lexer → Tokens → Parser → AST → Interpreter → 执行
```

## 组件详解

### 1. 词法分析器 (Lexer)

**位置**: `src/lexer/mod.rs`, `src/lexer/token.rs`

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

**位置**: `src/parser/mod.rs`

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

**位置**: `src/ast/mod.rs`

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

### 4. 解释器 (Interpreter)

**位置**: `src/interpreter/mod.rs`

**职责**:
- 遍历AST并执行
- 管理运行时环境和变量作用域
- 处理函数调用和返回值
- 执行算术和逻辑运算

**核心组件**:

#### 值类型 (Value)
- Integer(i64)
- Float(f64)
- String(String)
- Boolean(bool)
- Function
- Null

#### 环境 (Environment)
- 作用域栈管理
- 变量定义、查找和赋值
- 函数调用时的新作用域创建

#### 运行时错误处理
- UndefinedVariable
- TypeMismatch
- DivisionByZero
- InvalidOperation

## 数据流示例

### 示例代码
```zero
let x = 10;
let y = 20;
print(x + y);
```

### 处理流程

1. **Lexer输出**:
```
[Let, Identifier("x"), Equal, Integer("10"), Semicolon,
 Let, Identifier("y"), Equal, Integer("20"), Semicolon,
 Print, LeftParen, Identifier("x"), Plus, Identifier("y"), RightParen, Semicolon]
```

2. **Parser输出（AST）**:
```
Program {
  statements: [
    VarDeclaration { name: "x", mutable: false, initializer: Integer(10) },
    VarDeclaration { name: "y", mutable: false, initializer: Integer(20) },
    Print { value: Binary { left: Identifier("x"), op: Add, right: Identifier("y") } }
  ]
}
```

3. **Interpreter执行**:
- 创建变量 x = 10
- 创建变量 y = 20
- 计算 x + y = 30
- 输出 30

## 错误处理

### 词法错误
- 无效字符
- 未闭合的字符串

### 语法错误
- 意外的Token
- 缺少必需的Token（如分号、括号）
- 无效的表达式

### 运行时错误
- 未定义的变量
- 类型不匹配
- 除零错误

## 性能考虑

1. **Token缓存**: Parser保存Token向量以支持回溯
2. **作用域栈**: 使用栈结构管理作用域，支持O(1)的作用域进入/退出
3. **值克隆**: 当前实现使用值克隆，未来可优化为引用计数

## 扩展性

### 添加新的语言特性

1. **添加新Token**:
   - 在 `src/lexer/token.rs` 中添加新的TokenType
   - 在 `Lexer::tokenize()` 中添加识别逻辑

2. **添加新的AST节点**:
   - 在 `src/ast/mod.rs` 中添加新的Expr或Stmt变体

3. **添加新的语法规则**:
   - 在 `src/parser/mod.rs` 中添加解析方法

4. **添加解释逻辑**:
   - 在 `src/interpreter/mod.rs` 中添加执行逻辑

## 测试策略

### 单元测试
- Lexer: 测试Token生成
- Parser: 测试AST构建
- Interpreter: 测试表达式求值和语句执行

### 集成测试
- 完整程序的端到端测试
- 示例程序验证

### 测试覆盖
- 正常情况测试
- 边界情况测试
- 错误情况测试

## 未来优化方向

1. **性能优化**:
   - 字节码编译
   - JIT编译
   - 内存池

2. **功能扩展**:
   - 类型系统
   - 模块系统
   - 标准库
   - 错误恢复

3. **工具链**:
   - REPL
   - 调试器
   - 语言服务器协议(LSP)支持