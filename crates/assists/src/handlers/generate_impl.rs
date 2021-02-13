use syntax::ast::{self, AstNode, NameOwner};

use crate::{AssistContext, AssistId, AssistKind, Assists, utils::generate_impl_text};

// Assist: generate_impl
//
// Adds a new inherent impl for a type.
//
// ```
// struct Ctx<T: Clone> {
//     data: T,$0
// }
// ```
// ->
// ```
// struct Ctx<T: Clone> {
//     data: T,
// }
//
// impl<T: Clone> Ctx<T> {
//     $0
// }
// ```
pub(crate) fn generate_impl(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    let nominal = ctx.find_node_at_offset::<ast::Adt>()?;
    let name = nominal.name()?;
    let target = nominal.syntax().text_range();

    acc.add(
        AssistId("generate_impl", AssistKind::Generate),
        format!("Generate impl for `{}`", name),
        target,
        |edit| {
            let start_offset = nominal.syntax().text_range().end();
            match ctx.config.snippet_cap {
                Some(cap) => {
                    let snippet = generate_impl_text(&nominal, "    $0");
                    edit.insert_snippet(cap, start_offset, snippet);
                }
                None => {
                    let snippet = generate_impl_text(&nominal, "");
                    edit.insert(start_offset, snippet);
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_target};

    use super::*;

    #[test]
    fn test_add_impl() {
        check_assist(
            generate_impl,
            "struct Foo {$0}\n",
            "struct Foo {}\n\nimpl Foo {\n    $0\n}\n",
        );
        check_assist(
            generate_impl,
            "struct Foo<T: Clone> {$0}",
            "struct Foo<T: Clone> {}\n\nimpl<T: Clone> Foo<T> {\n    $0\n}",
        );
        check_assist(
            generate_impl,
            "struct Foo<'a, T: Foo<'a>> {$0}",
            "struct Foo<'a, T: Foo<'a>> {}\n\nimpl<'a, T: Foo<'a>> Foo<'a, T> {\n    $0\n}",
        );
        check_assist(
            generate_impl,
            r#"
            #[cfg(feature = "foo")]
            struct Foo<'a, T: Foo<'a>> {$0}"#,
            r#"
            #[cfg(feature = "foo")]
            struct Foo<'a, T: Foo<'a>> {}

            #[cfg(feature = "foo")]
            impl<'a, T: Foo<'a>> Foo<'a, T> {
                $0
            }"#,
        );

        check_assist(
            generate_impl,
            r#"
            #[cfg(not(feature = "foo"))]
            struct Foo<'a, T: Foo<'a>> {$0}"#,
            r#"
            #[cfg(not(feature = "foo"))]
            struct Foo<'a, T: Foo<'a>> {}

            #[cfg(not(feature = "foo"))]
            impl<'a, T: Foo<'a>> Foo<'a, T> {
                $0
            }"#,
        );
    }

    #[test]
    fn add_impl_target() {
        check_assist_target(
            generate_impl,
            "
struct SomeThingIrrelevant;
/// Has a lifetime parameter
struct Foo<'a, T: Foo<'a>> {$0}
struct EvenMoreIrrelevant;
",
            "/// Has a lifetime parameter
struct Foo<'a, T: Foo<'a>> {}",
        );
    }
}
