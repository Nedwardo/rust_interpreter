// fn equality() -> Expr{
//     Binary(
//         &lhs, 
//         Token {
//             token_
//         }
//     )
// }
//     Expr expr = comparison();

//     while (match(BANG_EQUAL, EQUAL_EQUAL)) {
//       Token operator = previous();
//       Expr right = comparison();
//       expr = new Expr.Binary(expr, operator, right);
//     }

//     return expr;