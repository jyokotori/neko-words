---
name: code-comments
description: 为代码生成清晰的注释和文档字符串。当用户要求添加注释、解释代码、生成 docstring、或说"帮我注释这段代码"时使用此 Skill。
---

# 代码注释 Skill

## 说明

为代码生成高质量的注释和文档字符串，遵循以下原则：

### 注释风格

1. **简洁明了**：注释应解释"为什么"而非"是什么"
2. **避免废话**：不要注释显而易见的代码
3. **保持同步**：注释必须与代码保持一致

### 文档字符串格式

根据语言选择合适的格式：

**Python** - 使用 Google 风格：
```python
def function_name(param1: str, param2: int) -> bool:
    """简短描述函数功能。

    更详细的说明（如果需要）。

    Args:
        param1: 参数1的说明
        param2: 参数2的说明

    Returns:
        返回值的说明

    Raises:
        ValueError: 何时抛出此异常
    """
```

**TypeScript/JavaScript** - 使用 JSDoc：
```typescript
/**
 * 简短描述函数功能
 *
 * @param param1 - 参数1的说明
 * @param param2 - 参数2的说明
 * @returns 返回值的说明
 * @throws {Error} 何时抛出此异常
 */
```

**Go** - 使用标准注释：
```go
// FunctionName 简短描述函数功能。
//
// 更详细的说明（如果需要）。
func FunctionName(param1 string, param2 int) (bool, error) {
```

### 行内注释原则

- 复杂算法：解释思路和步骤
- 业务逻辑：说明业务规则和原因
- 临时方案：标注 TODO 或 FIXME
- 性能优化：解释为什么这样写更快

### 不要注释的情况

- 变量名已经自解释的声明
- 简单的 getter/setter
- 框架约定的标准代码
- 已经很清晰的简单逻辑

## 示例

用户: "帮我注释这个函数"

```python
def calculate_discount(price, user_type, quantity):
    if user_type == "vip":
        base_discount = 0.2
    else:
        base_discount = 0.1

    if quantity >= 10:
        base_discount += 0.05

    return price * (1 - base_discount)
```

输出:

```python
def calculate_discount(price: float, user_type: str, quantity: int) -> float:
    """根据用户类型和购买数量计算折扣后价格。

    折扣规则：
    - VIP 用户基础折扣 20%，普通用户 10%
    - 批量购买（>=10件）额外 5% 折扣

    Args:
        price: 商品原价
        user_type: 用户类型，"vip" 或其他
        quantity: 购买数量

    Returns:
        折扣后的价格
    """
    # VIP 用户享受更高的基础折扣
    if user_type == "vip":
        base_discount = 0.2
    else:
        base_discount = 0.1

    # 批量购买奖励
    if quantity >= 10:
        base_discount += 0.05

    return price * (1 - base_discount)
```
