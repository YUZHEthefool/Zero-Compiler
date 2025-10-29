# Zero 语言类型系统设计

## 概述

Zero语言采用静态类型系统，提供编译期类型检查以提高代码安全性和性能。

## 类型系统特性

### 1. 基本类型

- `int` - 64位整数
- `float` - 64位浮点数
- `string` - 字符串
- `bool` - 布尔值
- `null` - 空类型

### 2. 类型注解语法

```zero
// 变量声明带类型注解
let x: int = 42;
let name: string = "Zero";
let flag: bool = true;
let price: float = 3.14;

// 函数参数类型注解
fn add(a: int, b: int) {
    return a + b;
}

fn greet(name: string) {
    return "Hello, " + name;
}

// 函数可以省略类型注解
fn print_message(msg) {
    print(msg);
}
```

### 3. 类型推导

支持简单的类型推导（局部类型推导）：

```zero
// 从初始值推导类型
let x = 42;           // 推导为 int
let y = 3.14;         // 推导为 float
let name = "Zero";    // 推导为 string
let flag = true;      // 推导为 bool

// 函数参数可以有类型注解
fn add(a: int, b: int) {
    return a + b;
}
```

### 4. 类型检查规则

#### 变量声明
- 必须有类型注解或可推导的初始值
- 类型注解和初始值类型必须匹配

#### 赋值
- 赋值表达式两侧类型必须匹配
- 不支持隐式类型转换

#### 函数调用
- 参数数量必须匹配
- 参数类型必须匹配
- 返回值类型必须符合声明

#### 运算符
- 算术运算符：`+`, `-`, `*`, `/`, `%`
  - 操作数必须是 `int` 或 `float`
  - 结果类型由操作数决定
  - `int` op `int` → `int`
  - `float` op `float` → `float`
  - `int` op `float` → `float` (自动提升)

- 比较运算符：`==`, `!=`, `<`, `<=`, `>`, `>=`
  - 操作数必须是同类型或可比较类型
  - 结果类型为 `bool`

- 逻辑运算符：`&&`, `||`, `!`
  - 操作数必须是 `bool`
  - 结果类型为 `bool`

- 字符串连接：`+`
  - 操作数必须都是 `string`
  - 结果类型为 `string`

#### 控制流
- `if` 条件必须是 `bool` 类型
- `while` 条件必须是 `bool` 类型

## 类型系统实现

### 1. 类型表示

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Null,
    Void,
    Function(FunctionType),
    Unknown,  // 用于类型推导
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
}
```

### 2. 类型检查器

位置：`src/type_checker/mod.rs`

职责：
- 遍历AST进行类型检查
- 维护符号表（变量和函数的类型信息）
- 报告类型错误
- 执行类型推导

### 3. 类型检查流程

```
AST → Type Checker → Typed AST → Compiler → Bytecode
```

### 4. 符号表

```rust
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Type>>,
}
```

- 支持作用域嵌套
- 记录变量和函数的类型信息
- 支持变量查找和类型查询

## 示例

### 正确的类型使用

```zero
// 基本类型
let x: int = 42;
let y: float = 3.14;
let name: string = "Zero";
let flag: bool = true;

// 函数定义
fn add(a: int, b: int) {
    return a + b;
}

fn multiply(x: float, y: float) {
    return x * y;
}

// 函数调用
let sum = add(10, 20);
let product = multiply(2.5, 4.0);

// 类型推导
let auto_int = 100;        // int
let auto_float = 2.718;    // float
let auto_string = "hello"; // string
```

### 类型错误示例

```zero
// 错误：类型不匹配
let x: int = 3.14;  // Error: Cannot assign float to int

// 错误：运算类型不匹配
let result = "hello" + 42;  // Error: Cannot add string and int

// 错误：参数类型不匹配
fn add(a: int, b: int) {
    return a + b;
}
let result = add(1.5, 2.5);  // Error: Expected int, got float

// 错误：条件类型不匹配
if 42 {  // Error: Expected bool, got int
    print("error");
}
```

## 类型错误信息

类型检查器将提供清晰的错误信息：

```
Type Error at line 5: 
  Expected type 'int', but got 'float'
  
Type Error: Argument count mismatch
  Function expects 2 arguments but got 3

Type Error: Argument type mismatch
  Function parameter 'x' expects type 'int', but got 'string'
  
Type Error at line 15:
  Cannot apply operator '+' to types 'string' and 'int'
```

## 未来扩展

### 1. 复合类型
- 数组类型：`[int]`, `[string]`
- 元组类型：`(int, string)`
- 结构体：`struct Point { x: int, y: int }`

### 2. 泛型
```zero
fn identity<T>(x: T) -> T {
    return x;
}
```

### 3. 类型别名
```zero
type Age = int;
type Name = string;
```

### 4. Option/Result 类型
```zero
fn divide(a: int, b: int) -> Result<int, string> {
    if b == 0 {
        return Err("Division by zero");
    }
    return Ok(a / b);
}
```

### 5. 接口/Trait
```zero
trait Printable {
    fn to_string() -> string;
}
```

## 实现阶段

### 阶段1：基本类型系统 ✅
- 基本类型定义
- 变量类型注解语法
- 函数参数类型注解
- 基本类型检查
- 类型错误报告
- 局部变量类型推导
- 表达式类型推导
- Unknown类型处理（用于无类型注解的函数参数）

### 阶段2：函数返回类型注解 📋
- 函数返回类型语法支持
- 返回类型检查
- 更严格的类型推导

### 阶段3：高级特性 📋
- 泛型支持
- 复合类型
- 类型别名
- Option/Result

## 参考

- [Rust Type System](https://doc.rust-lang.org/book/ch03-02-data-types.html)
- [TypeScript Type System](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html)
- [Hindley-Milner Type System](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system)