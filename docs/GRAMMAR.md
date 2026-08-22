# GRAMMAR.md — 정신뼝! v1.0 EBNF

```ebnf
program        ::= statement* ;

statement      ::= decl_stmt | assign_stmt | output_stmt | input_stmt
                 | return_stmt | throw_stmt | comment_stmt | if_stmt
                 | while_stmt | for_stmt | func_def | try_stmt
                 | call_stmt ;

decl_stmt      ::= "행정" "정신" identifier "!" ;
assign_stmt    ::= "뼝" "정신" identifier expression "!" ;
output_stmt    ::= "행정" "!" expression "!" ;
input_stmt     ::= "?" "행정" "정신" identifier "!" ;
return_stmt    ::= "행정" "뼝" expression "!" ;
throw_stmt     ::= "행정" "정병" expression "!" ;
comment_stmt   ::= "정병" token+ "!" ;
call_stmt      ::= "입원" identifier argument "!" ;

if_stmt        ::= "?" expression "멘헤라" "자의" statement* "동의" statement* "뼝" ;
while_stmt     ::= "우범" "?" expression "멘헤라" statement* "뼝" ;
for_stmt       ::= "우범" "소년" identifier expression expression "멘헤라" statement* "뼝" ;

func_def       ::= "입원" identifier param_list "멘헤라" statement* "뼝" ;
param_list     ::= ( "소년" identifier )* ;
try_stmt       ::= "보호" "멘헤라" statement* "정병" "멘헤라" statement* "뼝" ;

block          ::= "멘헤라" statement* "뼝" ;

expression     ::= logical_or ;
logical_or     ::= logical_and ( "?" logical_and "동의" logical_and )* ;
logical_and    ::= equality ( "?" equality )* ;
equality       ::= comparison ( "멘헤라" ( "자의" | "동의" ) comparison )* ;
comparison     ::= additive ( "멘헤라" ( "우범" | "소년" ) additive )* ;
additive       ::= multiplicative ( ( "뼝" | "정신" ) multiplicative )* ;
multiplicative ::= unary ( "원" unary )* ;
unary          ::= "!" unary | primary ;

primary        ::= identifier_ref | int_literal | str_literal | bool_literal
                 | won_literal | call_expr | grouped ;

grouped        ::= "멘헤라" expression "멘헤라" ;
identifier_ref ::= "정신" identifier ;
int_literal    ::= "멘헤라" bit+ "멘헤라" ;
str_literal    ::= "저엉신저엉신" bit+ "저엉신저엉신" ;
bool_literal   ::= "멘헤라" "입원" "멘헤라" | "멘헤라" "보호" "멘헤라" ;
won_literal    ::= "원"+ ;
call_expr      ::= "입원" identifier argument ;
argument       ::= additive ;
bit            ::= "정신" | "뼝" ;
identifier     ::= token ;
token          ::= 허용 15토큰 중 "!" "?" 제외 ;
```

## 파서 특수 규칙

1. `뼝` 단독은 블록 종료, `뼝 정신`은 대입 시작.
2. `입원` 뒤 `멘헤라`+비트는 호출 인자, `멘헤라`+문장은 함수 본문.
3. `멘헤라 자의`가 if 분기인지 `==`인지: 다음 토큰이 문장 시작이면 if.
