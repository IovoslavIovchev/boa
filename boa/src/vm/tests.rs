use super::*;
use crate::parser_expr;

#[test]
fn poc() {
    let src = r#"
const x = 7 + 4 + 1.1;
"#;

    let expr = parser_expr(src).expect("parsing failed");
    let instrs = Compiler::new().compile(&expr);

    let mut vm = VM::new(Realm::create());

    let res = vm.run(&instrs);

    assert_eq!(Ok(Value::new(ValueData::Number(12.1))), res);
}
