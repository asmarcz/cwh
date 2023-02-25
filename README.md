# Calculator with history (cwh)

A simple prefix notation integer calculator featuring
history of previous results.

## Usage

The calculator supports binary division, subtraction, multiplication
and addition using `/`, `-`, `*` and `+` respectively,
along with unary absolute value, factorial, negative,
predecessor, signum and successor using `abs`,
`fact`, `neg`, `pred`, `sgn` and `succ` respectively.

Previous results can be used by prefixing an index
in history with a `$` like `$0`.

Inputting a sole value or variable will push it on
top of history.

## Examples

Binary operations:
```
# + 3 2
5
# * 6 9
54
# / $1 $0
10
# - $2 + $0 $1
-49
```

Pushing to history:
```
# 13
13
# $0
13
# $1
13
```

Unary operations:
```
# abs -5
5
# neg $0
-5
# sgn $1
-1
# pred 7
6
# succ $3
7
# fact $4
5040
```
