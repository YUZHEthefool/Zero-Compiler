# Zero字节码文件格式规范

## 概述

Zero字节码文件（.zbc）是一种二进制格式，用于存储编译后的Zero程序。该格式设计用于快速加载和执行。

## 文件结构

```
+------------------+
| 文件头           |  (16 bytes)
+------------------+
| 常量池           |  (variable)
+------------------+
| 指令序列         |  (variable)
+------------------+
| 行号信息         |  (variable)
+------------------+
```

## 1. 文件头（Header）

文件头固定为16字节：

```
Offset | Size | Field           | Description
-------|------|-----------------|----------------------------------
0x00   | 4    | Magic           | 魔数: 0x5A45524F ("ZERO")
0x04   | 2    | Version Major   | 主版本号（目前为 0）
0x06   | 2    | Version Minor   | 次版本号（目前为 1）
0x08   | 4    | Constants Count | 常量池条目数量
0x0C   | 4    | Code Count      | 指令数量
```

## 2. 常量池（Constant Pool）

常量池包含程序使用的所有常量值。每个常量的格式：

```
+--------+
| Type   |  (1 byte)
+--------+
| Data   |  (variable)
+--------+
```

### 常量类型标识

| Type ID | Name     | Data Format                           |
|---------|----------|---------------------------------------|
| 0x01    | Integer  | 8 bytes (i64, little-endian)         |
| 0x02    | Float    | 8 bytes (f64, little-endian)         |
| 0x03    | String   | 4 bytes (length) + UTF-8 bytes       |
| 0x04    | Boolean  | 1 byte (0 = false, 1 = true)         |
| 0x05    | Array    | 4 bytes (length) + value indices     |
| 0x06    | Function | Function data (详见函数格式)          |
| 0x07    | Null     | 无数据                                |

### 函数常量格式

```
+------------------+
| Name Length      |  (4 bytes)
+------------------+
| Name (UTF-8)     |  (variable)
+------------------+
| Arity            |  (4 bytes)
+------------------+
| Locals Count     |  (4 bytes)
+------------------+
| Chunk Data       |  (nested chunk: constants + code + lines)
+------------------+
```

## 3. 指令序列（Code Section）

指令序列包含所有字节码指令。每条指令的格式：

```
+--------+
| OpCode |  (1 byte)
+--------+
| Args   |  (variable, depends on opcode)
+--------+
```

### 操作码表

| OpCode | Name            | Args                    | Description              |
|--------|-----------------|-------------------------|--------------------------|
| 0x00   | LoadConst       | index: u32 (4 bytes)   | 加载常量                  |
| 0x01   | LoadNull        | 无                      | 加载null值                |
| 0x02   | LoadLocal       | slot: u32 (4 bytes)    | 加载局部变量              |
| 0x03   | StoreLocal      | slot: u32 (4 bytes)    | 存储局部变量              |
| 0x04   | LoadGlobal      | index: u32 (4 bytes)   | 加载全局变量              |
| 0x05   | StoreGlobal     | index: u32 (4 bytes)   | 存储全局变量              |
| 0x10   | Add             | 无                      | 加法                      |
| 0x11   | Subtract        | 无                      | 减法                      |
| 0x12   | Multiply        | 无                      | 乘法                      |
| 0x13   | Divide          | 无                      | 除法                      |
| 0x14   | Modulo          | 无                      | 取模                      |
| 0x15   | Negate          | 无                      | 取负                      |
| 0x20   | Equal           | 无                      | 相等比较                  |
| 0x21   | NotEqual        | 无                      | 不等比较                  |
| 0x22   | Greater         | 无                      | 大于比较                  |
| 0x23   | GreaterEqual    | 无                      | 大于等于比较              |
| 0x24   | Less            | 无                      | 小于比较                  |
| 0x25   | LessEqual       | 无                      | 小于等于比较              |
| 0x30   | Not             | 无                      | 逻辑非                    |
| 0x31   | And             | 无                      | 逻辑与                    |
| 0x32   | Or              | 无                      | 逻辑或                    |
| 0x40   | Jump            | offset: u32 (4 bytes)  | 无条件跳转                |
| 0x41   | JumpIfFalse     | offset: u32 (4 bytes)  | 条件跳转（假）            |
| 0x42   | JumpIfTrue      | offset: u32 (4 bytes)  | 条件跳转（真）            |
| 0x43   | Loop            | offset: u32 (4 bytes)  | 循环跳转                  |
| 0x50   | Call            | argc: u32 (4 bytes)    | 函数调用                  |
| 0x51   | Return          | 无                      | 返回                      |
| 0x60   | NewArray        | size: u32 (4 bytes)    | 创建数组                  |
| 0x61   | ArrayGet        | 无                      | 获取数组元素              |
| 0x62   | ArraySet        | 无                      | 设置数组元素              |
| 0x63   | ArrayLen        | 无                      | 获取数组长度              |
| 0x70   | Pop             | 无                      | 弹出栈顶                  |
| 0x71   | Dup             | 无                      | 复制栈顶                  |
| 0xF0   | Print           | 无                      | 打印                      |
| 0xFF   | Halt            | 无                      | 停止执行                  |

## 4. 行号信息（Line Info）

行号信息用于错误报告和调试。格式：

```
每条指令对应一个行号: u32 (4 bytes, little-endian)
```

行号数量应等于指令数量。

## 示例

### 简单程序

源代码：
```zero
let x: int = 42;
print(x);
```

字节码文件结构：
```
Header:
  Magic: 5A 45 52 4F
  Version: 00 00 00 01
  Constants: 03 00 00 00  (3个常量)
  Code: 05 00 00 00       (5条指令)

Constants:
  [0] Integer: 01 2A 00 00 00 00 00 00 00  (42)
  [1] String: 03 01 00 00 00 78            ("x")
  [2] String: 03 01 00 00 00 78            ("x")

Code:
  LoadConst 0      : 00 00 00 00 00
  StoreGlobal 1    : 05 01 00 00 00
  Pop              : 70
  LoadGlobal 2     : 04 02 00 00 00
  Print            : F0

Lines:
  01 00 00 00  (line 1)
  01 00 00 00  (line 1)
  01 00 00 00  (line 1)
  02 00 00 00  (line 2)
  02 00 00 00  (line 2)
```

## 文件扩展名

- `.zbc` - Zero Bytecode（已编译的字节码）
- `.zero` - Zero源代码

## 版本兼容性

当前版本：0.1

- 主版本号变更表示不兼容的格式更改
- 次版本号变更表示向后兼容的功能添加

## 字节序

所有多字节整数使用**小端序（Little-Endian）**存储。

## 校验

建议在文件末尾添加CRC32校验和（可选，目前版本未实现）。