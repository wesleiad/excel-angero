use gridcraft_formula::{FormulaLocale, parse, parse_input, print, print_input};

fn input(src: &str) -> String {
    print(&parse_input(src, FormulaLocale::Pt, |_| false).unwrap())
}

#[test]
fn portuguese_aliases_and_argument_separators() {
    for (src, expected) in [
        ("=soma(A1:A10)", "SUM(A1:A10)"),
        ("MÉDIA(A1:A10)", "AVERAGE(A1:A10)"),
        ("SE(A1>0;\"Sí\";\"No\")", "IF(A1>0,\"Sí\",\"No\")"),
        ("CONT.NÚM(A1:A10)", "COUNT(A1:A10)"),
        ("SOMASE(A1:A10;\">0\")", "SUMIF(A1:A10,\">0\")"),
        ("CONT.SE(A1:A10;\">0\")", "COUNTIF(A1:A10,\">0\")"),
        ("PROCV(A2;D:E;2;FALSO)", "VLOOKUP(A2,D:E,2,FALSE)"),
        ("HOJE()", "TODAY()"),
        ("=verdadeiro", "TRUE"),
        ("=falso", "FALSE"),
        ("VERDADEIRO()", "TRUE()"),
        ("FALSO()", "FALSE()"),
        ("SOMA(1;SUM(2;3);MÉDIA(4;6))", "SUM(1,SUM(2,3),AVERAGE(4,6))"),
        ("SE(VERDADEIRO;;)", "IF(TRUE,,)"),
        ("LAMBDA(x;x+1)(2)", "LAMBDA(x,x+1)(2)"),
    ] {
        assert_eq!(input(src), expected, "{src}");
    }
}

#[test]
fn portuguese_input_preserves_reference_and_array_syntax() {
    for (src, expected) in [
        ("SOMA({1\\2;3\\4};5)", "SUM({1,2;3,4},5)"),
        ("{VERDADEIRO\\FALSO;TRUE\\FALSE}", "{TRUE,FALSE;TRUE,FALSE}"),
        ("SUM((A1:A3;C1:C3))", "SUM((A1:A3,C1:C3))"),
        ("\"SOMA;FALSO\"&\"a\"\"b\"", "\"SOMA;FALSO\"&\"a\"\"b\""),
        ("'SOMA;FALSO'!A1+SOMA!B2+SOMA!FALSO", "'SOMA;FALSO'!A1+SOMA!B2+SOMA!FALSO"),
        ("SOMA[CONT.SE]+FALSO[SE]+CONT.NÚM", "SOMA[CONT.SE]+FALSO[SE]+CONT.NÚM"),
    ] {
        assert_eq!(input(src), expected, "{src}");
    }
    for bad in [r"SOMA({1;2\3})", "{SOMA(A1)}", "{FALSO;}"] {
        assert!(parse_input(bad, FormulaLocale::Pt, |_| false).is_err(), "{bad}");
    }
    assert_eq!(print(&parse("SOMA(1,2)+FALSO").unwrap()), "SOMA(1,2)+FALSO");
    assert!(parse("SUM(1;2)").is_err());
    assert!(parse("{VERDADEIRO}").is_err());
    // Pratt parsing can produce a deep left-hand AST without deep parser recursion.
    let long_chain = "FALSO+".repeat(300) + "VERDADEIRO";
    assert!(parse_input(&long_chain, FormulaLocale::Pt, |_| false).is_err());
    assert!(print_input(&parse(&long_chain).unwrap(), FormulaLocale::Pt, |_| false).is_err());
}

#[test]
fn portuguese_input_preserves_defined_and_local_names() {
    let known = |name: &str| ["SOMA", "FALSO"].iter().any(|n| n.eq_ignore_ascii_case(name));
    let expr = parse_input("SOMA(2;3)+FALSO+SE(VERDADEIRO;4;5)", FormulaLocale::Pt, known).unwrap();
    assert_eq!(print(&expr), "SOMA(2,3)+FALSO+IF(TRUE,4,5)");
    for (src, expected) in [
        ("LET(FALSO;7;FALSO)", "LET(FALSO,7,FALSO)"),
        ("LET(falso;FALSO;SE(falso;1;2))", "LET(falso,FALSE,IF(falso,1,2))"),
        ("LET(SOMA;LAMBDA(x;x+1);SOMA(2))", "LET(SOMA,LAMBDA(x,x+1),SOMA(2))"),
        ("LET(falso;7;valor;FALSO;valor)", "LET(falso,7,valor,FALSO,valor)"),
        ("LAMBDA(FALSO;FALSO)(7)", "LAMBDA(FALSO,FALSO)(7)"),
        ("LAMBDA(SOMA;SOMA(2))(LAMBDA(x;x+1))", "LAMBDA(SOMA,SOMA(2))(LAMBDA(x,x+1))"),
        ("LET(FALSO;7;FALSO)+FALSO", "LET(FALSO,7,FALSO)+FALSE"),
        ("LET(FALSO;7;LAMBDA(x;FALSO+x)(1))", "LET(FALSO,7,LAMBDA(x,FALSO+x)(1))"),
    ] {
        assert_eq!(input(src), expected, "{src}");
    }
}

#[test]
fn locale_controls_decimal_list_array_separators_and_display() {
    let pt = FormulaLocale::Pt;
    let en = FormulaLocale::En;
    let expr = parse_input("SE(VERDADEIRO;SOMA(1,5;2,25);0)", pt, |_| false).unwrap();
    assert_eq!(print(&expr), "IF(TRUE,SUM(1.5,2.25),0)");
    assert_eq!(print_input(&expr, pt, |_| false).unwrap(), "SE(VERDADEIRO;SOMA(1,5;2,25);0)");
    assert_eq!(print_input(&expr, en, |_| false).unwrap(), print(&expr));
    assert!(parse_input("SUM(1;2)", en, |_| false).is_err());
    assert_eq!(print(&parse_input("SOMA(1,2)+FALSO", en, |_| false).unwrap()), "SOMA(1,2)+FALSO");
    assert!(parse_input("SOMA(1.5;2)", pt, |_| false).is_err());
    let expr = parse("SUM({1.5,2.25;3,4})+SUM((A1:A3,C1:C3))").unwrap();
    let shown = print_input(&expr, pt, |_| false).unwrap();
    assert_eq!(print(&parse_input(&shown, pt, |_| false).unwrap()), print(&expr));
    let expr = parse("LET(SOMA,LAMBDA(x,x+1),SUM(SOMA(2),FALSE))").unwrap();
    let shown = print_input(&expr, pt, |n| n == "FALSO").unwrap();
    assert_eq!(shown, "LET(SOMA;LAMBDA(x;x+1);SUM(SOMA(2);FALSE))");
    assert_eq!(print(&parse_input(&shown, pt, |n| n == "FALSO").unwrap()), print(&expr));
}

#[test]
fn portuguese_display_round_trips_text_containing_separators() {
    let pt = FormulaLocale::Pt;
    let expr = parse(r#"IF(A1="a,b;c",CONCAT("1.5",",",";"),SUM(0.25,A1:A3 B2))"#).unwrap();
    let shown = print_input(&expr, pt, |_| false).unwrap();
    assert_eq!(shown, r#"SE(A1="a,b;c";CONCAT("1.5";",";";");SOMA(0,25;A1:A3 B2))"#);
    assert_eq!(print(&parse_input(&shown, pt, |_| false).unwrap()), print(&expr));
}

#[test]
fn portuguese_accented_names_round_trip() {
    let pt = FormulaLocale::Pt;
    for (src, expected) in [
        ("=média(1;2,5)", "AVERAGE(1,2.5)"),
        ("MÁXIMO(A1:A3)+MÍNIMO(B1:B3)", "MAX(A1:A3)+MIN(B1:B3)"),
        ("SE(ÉCÉL.VAZIA(A1);\"\";PROCX(A1;B:B;C:C))", "IF(ISBLANK(A1),\"\",XLOOKUP(A1,B:B,C:C))"),
        ("CONT.SES(A:A;\">0\";B:B;\"<5\")", "COUNTIFS(A:A,\">0\",B:B,\"<5\")"),
        ("SEERRO(PROCV(A2;D:E;2;FALSO);0)", "IFERROR(VLOOKUP(A2,D:E,2,FALSE),0)"),
        ("E(VERDADEIRO;OU(FALSO;NÃO(FALSO)))", "AND(TRUE,OR(FALSE,NOT(FALSE)))"),
    ] {
        assert_eq!(input(src), expected, "{src}");
        let shown = print_input(&parse(expected).unwrap(), pt, |_| false).unwrap();
        assert_eq!(input(&shown), expected, "{shown}");
    }
}
