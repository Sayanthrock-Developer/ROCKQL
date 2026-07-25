# RockQL MVP language reference

RockQL v0.1 accepts a source followed by a top-to-bottom sequence of transformations. Transformations may be separated by newlines or `|` characters.

```rockql
from employees
filter salary > 50_000
derive yearly_salary = salary * 12
sort {-yearly_salary}
take 10
```

## Transformations

### `from`

Selects the source relation. It must be the first statement.

```rockql
from analytics.employees
```

### `filter`

Adds a SQL `WHERE` condition.

```rockql
filter active == true and salary >= 50_000
```

### `select`

Chooses output expressions. Braces are optional.

```rockql
select {id, name, email}
```

### `derive`

Adds a computed output column.

```rockql
derive yearly_salary = salary * 12
```

### `sort`

Sorts output rows. Prefix an expression with `-` for descending or `+` for ascending.

```rockql
sort {-yearly_salary, +name}
```

### `take`

Limits the number of output rows.

```rockql
take 20
```

## MVP expression normalisation

The compiler translates `==` to SQL `=`, normalises `and`, `or`, `not`, booleans and `null`, removes numeric separators, and converts top-level `??` expressions into `COALESCE(left, right)`.

The v0.1 parser intentionally keeps expressions lightweight. A typed expression tree, functions, dates, ranges, joins, grouping and aggregation belong to later milestones.
