use quagmire_compiler::ast::*;
use quagmire_compiler::{Diagnostic, Parser, SourceMap};

fn parse(text: &str) -> Result<Module, Diagnostic> {
    let mut sources = SourceMap::new();
    let id = sources.add("test.qg", text);
    let result = Parser::new(&sources, id).unwrap().parse();
    if let Err(error) = &result {
        sources.slice(error.span).unwrap();
        assert!(error.render(&sources).unwrap().contains("test.qg:"));
    }
    result
}

fn constant(text: &str) -> Expression {
    let module = parse(&format!("const result = {text};")).unwrap();
    let DeclarationKind::Constant(value) = module.declarations.into_iter().next().unwrap().kind
    else {
        panic!("constant")
    };
    value.value
}

#[test]
fn actual_language_files_parse_into_spanned_modules() {
    let mut sources = SourceMap::new();
    for file in ["app.qg", "counter.qg"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/language")
            .join(file);
        let id = sources.load(&path).unwrap();
        let module = Parser::new(&sources, id)
            .unwrap()
            .parse()
            .unwrap_or_else(|error| panic!("{}", error.render(&sources).unwrap()));
        assert_eq!(module.declarations.len(), 1);
        assert!(module.declarations[0].exported);
        let DeclarationKind::Component(component) = &module.declarations[0].kind else {
            panic!("component")
        };
        assert!(
            sources
                .slice(component.span)
                .unwrap()
                .starts_with("component ")
        );
        assert!(
            sources
                .slice(module.span)
                .unwrap()
                .starts_with(if file == "app.qg" { "import" } else { "export" })
        );
    }
}

#[test]
fn expression_precedence_associativity_and_postfix_are_structural() {
    let ExpressionKind::Binary {
        operator: BinaryOperator::Subtract,
        left,
        right,
    } = constant("1 + 2 * 3 - 4").kind
    else {
        panic!("subtract")
    };
    assert!(matches!(right.kind, ExpressionKind::Number(4.0)));
    let ExpressionKind::Binary {
        operator: BinaryOperator::Add,
        right,
        ..
    } = left.kind
    else {
        panic!("add")
    };
    assert!(matches!(
        right.kind,
        ExpressionKind::Binary {
            operator: BinaryOperator::Multiply,
            ..
        }
    ));
    assert!(matches!(
        constant("!ready || a === b && c < d").kind,
        ExpressionKind::Binary {
            operator: BinaryOperator::Or,
            ..
        }
    ));
    assert!(matches!(
        constant("fetch(1,)[0].name").kind,
        ExpressionKind::Member { .. }
    ));
    assert!(matches!(
        constant("(1 + 2) * 3").kind,
        ExpressionKind::Binary {
            operator: BinaryOperator::Multiply,
            ..
        }
    ));
    assert!(parse("const result = a < b < c;").is_err());
    assert!(parse("const result = (a < b) < c;").is_ok());
}

#[test]
fn declarations_parameters_imports_and_statements_are_preserved() {
    let module = parse(
        r#"
        import { One, Two, } from "./helpers.qg";
        export function choose(value, fallback = 0,) {
            let result = fallback;
            const step = 1;
            if (value < 0) { return fallback; }
            else if (value > 10) { return 10; }
            else { result = value; }
            for (item in [1, 2,]) { consume(item); }
            return result;
        }
        const record = { "name": "Justin", active: true, empty: null, };
    "#,
    )
    .unwrap();
    assert_eq!(module.imports[0].names.len(), 2);
    assert_eq!(module.imports[0].path, "./helpers.qg");
    let DeclarationKind::Function(function) = &module.declarations[0].kind else {
        panic!("function")
    };
    assert_eq!(function.parameters.len(), 2);
    assert!(function.parameters[1].default.is_some());
    assert_eq!(function.body.statements.len(), 5);
    let StatementKind::If(condition) = &function.body.statements[2].kind else {
        panic!("if")
    };
    assert_eq!(condition.branches.len(), 2);
    assert!(condition.otherwise.is_some());
    assert!(matches!(
        function.body.statements[3].kind,
        StatementKind::For { .. }
    ));
}

#[test]
fn view_modes_allow_nested_objects_comparisons_and_literal_comment_markers() {
    let module = parse(
        r#"component Example() {
        view { <section title={{ name: "}" }.name}>
            // literal comment markers
            {count < 2}
            if (ready) { <p>Yes &amp; &#x1F642;</p> }
            else if (waiting) { <p>Wait</p> } else { <p>No</p> }
            /* also literal, not an unterminated code comment
            <input value="&amp;" />
        </section> }
    }"#,
    )
    .unwrap();
    let DeclarationKind::Component(component) = &module.declarations[0].kind else {
        panic!("component")
    };
    assert_eq!(component.view.root.name.text, "section");
    assert!(matches!(
        component.view.root.attributes[0].value,
        AttributeValue::Expression(_)
    ));
    assert!(component.view.root.children.iter().any(|node| matches!(&node.kind, ViewNodeKind::If { branches, otherwise } if branches.len() == 2 && otherwise.is_some())));
    assert!(component.view.root.children.iter().any(
        |node| matches!(&node.kind, ViewNodeKind::Text(text) if text.contains("/* also literal"))
    ));
}

#[test]
fn nested_view_controls_and_component_loop_keys_parse() {
    assert!(
        parse(
            r#"component App() { view { <main>
        if (ready) {
            for (item in items) { <Counter key={item.id} initial={item.initial} /> }
        } else { }
        for (group in groups) {
            <section key={group.id}>
                for (item in group.items) { <p key={item.id}>{item.name}</p> }
            </section>
        }
    </main> } }"#
        )
        .is_ok()
    );
}

#[test]
fn invalid_syntax_and_unsupported_forms_produce_diagnostics() {
    for source in [
        "state outside = 0;",
        "import {} from \"./empty.qg\";",
        "const x = 0; import { X } from \"./late.qg\";",
        "const x = { name };",
        "const x = [1,,2];",
        "const x = (a = 1);",
        "function f() { value.name = 1; }",
        "function f() { value.name; }",
        "function f() { f() + g(); }",
        "function f() { return 1 }",
        "function f(optional = 1, required) {}",
        "function f() { state count = 0; }",
        "component app() { view { <div /> } }",
        "component App() { state count; view { <p /> } }",
        "component App() { view { <p /> <p /> } }",
        "component App() { view { } }",
        "component App() { view { <p /> } state x = 1; }",
        "component App() { view { <div><span></div> } }",
        "component App() { view { <input> } }",
        "component App() { view { <Counter></Counter> } }",
        "component App() { view { <p class=\"a\" class=\"b\" /> } }",
        "component App() { view { <p value=3 /> } }",
        "component App() { view { <main>for (item in items) { <p /> }</main> } }",
        "component App() { view { <main>for (item in items) { <p key=\"constant\" /> }</main> } }",
        "component App() { view { <main>for (item in items) { <p key={item.id} /> <p /> }</main> } }",
        "component App() { view { <main>{}</main> } }",
        "component App() { view { <div>text",
        "function f() { if ready { return; } }",
    ] {
        assert!(parse(source).is_err(), "unexpectedly accepted {source}");
    }
}

#[test]
fn lexical_failures_keep_their_original_codes() {
    assert_eq!(parse("const x = 01;").unwrap_err().code, "QG1004");
    assert_eq!(parse("const x = \"unfinished").unwrap_err().code, "QG1002");
    assert_eq!(
        parse("component App() { view { <p>&unknown;</p> } }")
            .unwrap_err()
            .code,
        "QG1007"
    );
}

#[test]
fn empty_modules_whitespace_comments_and_bare_returns_work() {
    assert!(parse(" /* module */ \n").unwrap().declarations.is_empty());
    assert!(parse("function f() { return; }").is_ok());
    assert!(parse("function f() { (f()); }").is_ok());
    assert!(parse("component App() { view { <p> </p> } }").is_ok());
    let mut sources = SourceMap::new();
    let id = sources.add("unused.qg", "");
    assert!(Parser::new(&SourceMap::new(), id).is_err());
}

#[test]
fn parser_limits_recursive_and_iterative_tree_depth_without_panicking() {
    for expression in [
        format!("{}1{}", "(".repeat(200), ")".repeat(200)),
        format!("{}ready", "!".repeat(200)),
        (0..200).map(|_| "1").collect::<Vec<_>>().join(" + "),
        format!("value{}", ".name".repeat(200)),
    ] {
        assert_eq!(
            parse(&format!("const x = {expression};")).unwrap_err().code,
            "QG2004"
        );
    }
    let nested = format!(
        "component App() {{ view {{ {}{} }} }}",
        "<div>".repeat(200),
        "</div>".repeat(200)
    );
    assert_eq!(parse(&nested).unwrap_err().code, "QG2004");
}
