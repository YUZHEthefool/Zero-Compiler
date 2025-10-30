# Zero 语言数组系统设计

## 概述

Zero语言将支持动态大小的数组，提供类型安全的数组操作。数组是同类型元素的有序集合。

## 数组语法

### 1. 数组类型注解

```zero
// 数组类型使用 [元素类型] 表示
let numbers: [int] = [1, 2, 3, 4, 5];
let names: [string] = ["Alice", "Bob", "Charlie"];
let flags: [bool] = [true, false, true];
let matrix: [[int]] = [[1, 2], [3, 4]];  // 二维数组

// 空数组需要类型注解
let empty: [int] = [];
```

### 2. 数组字面量

```zero
// 数组字面量使用方括号
let numbers = [1, 2, 3, 4, 5];           // 推导为 [int]
let mixed = [1, 2, 3];                   // 推导为 [int]
let names = ["Alice", "Bob"];            // 推导为 [string]

// 嵌套数组
let matrix = [[1, 2, 3], [4, 5, 6]];    // 推导为 [[int]]
```

### 3. 数组索引访问

```zero
let numbers = [10, 20, 30, 40, 50];

// 读取元素
let first = numbers[0];      // 10
let third = numbers[2];      // 30

// 修改元素
numbers[0] = 100;            // numbers 变为 [100, 20, 30, 40, 50]
numbers[2] = numbers[1] + 10; // numbers[2] = 30

// 负索引（从末尾访问）
let last = numbers[-1];      // 最后一个元素
let second_last = numbers[-2]; // 倒数第二个元素
```

### 4. 数组切片

```zero
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// 切片语法: array[start:end]
// 包含start，不包含end
let slice1 = numbers[2:5];    // [3, 4, 5]
let slice2 = numbers[:3];     // [1, 2, 3] - 从开始到索引3
let slice3 = numbers[5:];     // [6, 7, 8, 9, 10] - 从索引5到末尾
let slice4 = numbers[:];      // [1, 2, ..., 10] - 完整副本

// 负索引切片
let slice5 = numbers[-3:];    // 最后3个元素
let slice6 = numbers[:-2];    // 除了最后2个的所有元素
```

### 5. 数组内置方法

```zero
let numbers = [3, 1, 4, 1, 5];

// len() - 获取数组长度
let length = len(numbers);    // 5

// push() - 在末尾添加元素
numbers.push(9);              // [3, 1, 4, 1, 5, 9]

// pop() - 移除并返回最后一个元素
let last = numbers.pop();     // 返回 9, numbers 变为 [3, 1, 4, 1, 5]

// insert() - 在指定位置插入元素
numbers.insert(2, 99);        // [3, 1, 99, 4, 1, 5]

// remove() - 移除指定位置的元素
numbers.remove(1);            // [3, 99, 4, 1, 5]

// contains() - 检查元素是否存在
let has_four = numbers.contains(4);  // true

// indexOf() - 查找元素的索引
let index = numbers.indexOf(4);      // 2 (如果不存在返回-1)

// reverse() - 反转数组
numbers.reverse();            // [5, 1, 4, 99, 3]

// sort() - 排序数组
numbers.sort();               // [1, 3, 4, 5, 99]

// map() - 映射操作
let doubled = numbers.map(fn(x) { return x * 2; });

// filter() - 过滤操作
let evens = numbers.filter(fn(x) { return x % 2 == 0; });

// reduce() - 归约操作
let sum = numbers.reduce(fn(acc, x) { return acc + x; }, 0);
```

## 数组类型系统

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
    Array(Box<Type>),        // 数组类型
    Function(FunctionType),
    Unknown,
}
```

### 2. 类型检查规则

#### 数组字面量
- 所有元素必须是同一类型
- 空数组需要显式类型注解
- 嵌套数组的内层数组必须类型一致

#### 数组索引
- 索引必须是 `int` 类型
- 索引访问返回数组元素类型
- 越界访问将产生运行时错误

#### 数组赋值
- 赋值的值必须与数组元素类型匹配
- 不能改变数组的元素类型

#### 数组操作
- `len()` 返回 `int`
- `push()` 参数类型必须与数组元素类型匹配
- `pop()` 返回数组元素类型
- `contains()` 参数类型必须与数组元素类型匹配，返回 `bool`

## 实现策略

### 阶段1: 基本数组支持 🚧
- 数组类型定义
- 数组字面量语法
- 数组索引访问（读取和写入）
- `len()` 内置函数
- 基本类型检查

### 阶段2: 数组方法 📋
- `push()` 和 `pop()`
- `insert()` 和 `remove()`
- `contains()` 和 `indexOf()`
- `reverse()` 和 `sort()`

### 阶段3: 高级特性 📋
- 数组切片
- `map()`, `filter()`, `reduce()`
- 多维数组完整支持
- 数组解构

## 字节码扩展

需要添加的新操作码：

```rust
pub enum OpCode {
    // ... 现有操作码 ...
    
    // 数组操作
    NewArray,              // 创建新数组
    ArrayGet,              // 获取数组元素
    ArraySet,              // 设置数组元素
    ArrayLen,              // 获取数组长度
    ArrayPush,             // 在末尾添加元素
    ArrayPop,              // 移除末尾元素
    ArrayInsert,           // 在指定位置插入
    ArrayRemove,           // 移除指定位置元素
    ArraySlice,            // 数组切片
}
```

## VM 值类型扩展

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Array(Vec<Value>),     // 数组值
}
```

## 示例代码

### 基础数组操作
```zero
// 创建和初始化
let numbers: [int] = [1, 2, 3, 4, 5];
let names = ["Alice", "Bob", "Charlie"];

// 访问和修改
print(numbers[0]);        // 1
numbers[0] = 10;
print(numbers[0]);        // 10

// 长度
print(len(numbers));      // 5
```

### 数组遍历
```zero
let numbers = [1, 2, 3, 4, 5];

// for循环遍历
for i in 0..len(numbers) {
    print(numbers[i]);
}

// foreach循环（未来特性）
for num in numbers {
    print(num);
}
```

### 多维数组
```zero
let matrix: [[int]] = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
];

// 访问元素
print(matrix[0][0]);      // 1
print(matrix[1][2]);      // 6

// 修改元素
matrix[1][1] = 50;
```

### 数组作为函数参数
```zero
fn sum(numbers: [int]) -> int {
    let total = 0;
    for i in 0..len(numbers) {
        total = total + numbers[i];
    }
    return total;
}

let nums = [1, 2, 3, 4, 5];
let result = sum(nums);
print(result);  // 15
```

## 错误处理

### 类型错误
```zero
// 错误：混合类型
let mixed = [1, "hello", true];  // Error: Array elements must be same type

// 错误：类型不匹配
let numbers: [int] = ["hello"];  // Error: Expected [int], got [string]

// 错误：索引类型错误
let x = numbers["hello"];        // Error: Index must be int
```

### 运行时错误
```zero
let numbers = [1, 2, 3];

// 越界访问
let x = numbers[10];             // Runtime Error: Index out of bounds

// 空数组pop
let empty: [int] = [];
let x = empty.pop();             // Runtime Error: Cannot pop from empty array
```

## 性能考虑

1. **动态大小**: 数组使用 Vec<Value> 实现，支持动态增长
2. **边界检查**: 所有索引访问都进行边界检查
3. **引用语义**: 数组作为引用传递（避免不必要的复制）
4. **内存管理**: 依赖Rust的所有权系统管理内存

## 未来扩展

1. **固定大小数组**: `[int; 5]` - 编译期已知大小
2. **数组推导**: `[x * 2 for x in 0..10]`
3. **模式匹配**: `match arr { [first, ...rest] => ... }`
4. **并行操作**: 并行的 map/filter/reduce
5. **不可变数组**: 优化只读场景的性能

## 参考

- [Rust Vec](https://doc.rust-lang.org/std/vec/struct.Vec.html)
- [Python Lists](https://docs.python.org/3/tutorial/datastructures.html)
- [JavaScript Arrays](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array)