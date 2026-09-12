// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export ordinary executable examples without the retired audit prerequisites.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::cli::Error;
use crate::context::Context;

#[path = "authoring/obligations.rs"]
mod obligations;
#[path = "authoring/spine.rs"]
mod spine;
#[path = "authoring/state_form.rs"]
mod state_form;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Edit {
    pub(crate) before: String,
    pub(crate) after: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Base {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) base: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) edits: Vec<Edit>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Case {
    pub(crate) id: String,
    pub(crate) base: String,
    pub(crate) fixtures: Vec<String>,
    pub(crate) pins: Vec<String>,
    pub(crate) edits: Vec<Edit>,
    pub(crate) scan: bool,
}

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Export {
    pub(crate) bases: BTreeMap<String, Base>,
    pub(crate) cases: Vec<Case>,
}

impl Export {
    pub(crate) fn new() -> Self {
        let mut value = Self::default();
        value.bases.insert(
            "live".into(),
            Base {
                path: Some("new-book-plans/constitution.nibli".into()),
                ..Base::default()
            },
        );
        value
    }

    pub(crate) fn add_case(
        &mut self,
        context: &Context,
        id: &str,
        base: &str,
        fixture: &str,
        pins: &[(&str, &str)],
        edits: Vec<Edit>,
        scan: bool,
    ) -> Result<(), Error> {
        if self.cases.iter().any(|case| case.id == id) {
            return Err(Error::new(format!("duplicate exported case {id}")));
        }
        if id
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-_/.".contains(&byte))
        {
            return Err(Error::new(format!("invalid exported case id {id:?}")));
        }
        let directory = format!("tests/pins/{id}");
        std::fs::create_dir_all(context.path(&directory))?;
        let mut fixtures = Vec::new();
        if !fixture.trim().is_empty() {
            let path = format!("{directory}/fixture.nibli");
            write_source(&context.path(&path), fixture)?;
            fixtures.push(path);
        }
        let mut paths = Vec::new();
        for (name, content) in pins {
            if name.contains('/') || name.contains('\\') || *name == "." || *name == ".." {
                return Err(Error::new(format!("invalid pin filename {name:?}")));
            }
            let path = format!("{directory}/{name}.pins.nibli");
            write_source(&context.path(&path), content)?;
            paths.push(path);
        }
        self.cases.push(Case {
            id: id.into(),
            base: base.into(),
            fixtures,
            pins: paths,
            edits,
            scan,
        });
        Ok(())
    }

    pub(crate) fn write(&self, path: &Path) -> Result<(), Error> {
        let mut text =
            serde_json::to_string_pretty(self).map_err(|error| Error::new(error.to_string()))?;
        text.push('\n');
        std::fs::write(path, text)?;
        Ok(())
    }
}

pub(crate) fn apply_edits(source: &str, edits: &[Edit]) -> Result<String, Error> {
    let mut source = source.to_owned();
    for edit in edits {
        if edit.before.is_empty() {
            source.push_str(&edit.after);
        } else {
            let count = source.matches(&edit.before).count();
            if count != 1 {
                return Err(Error::new(format!(
                    "source edit needs one occurrence, found {count}: {:?}",
                    edit.before
                )));
            }
            source = source.replacen(&edit.before, &edit.after, 1);
        }
    }
    Ok(source)
}

fn write_source(path: &Path, content: &str) -> Result<(), Error> {
    if content
        .lines()
        .any(|line| line.starts_with("# SPDX-License-Identifier:"))
    {
        std::fs::write(path, content)?;
    } else {
        std::fs::write(
            path,
            format!("# SPDX-License-Identifier: MIT OR Apache-2.0\n{content}"),
        )?;
    }
    Ok(())
}

fn merge_family(
    inventory: &mut serde_json::Value,
    family: &str,
    export: Export,
) -> Result<(), Error> {
    let prefix = format!("{family}/");
    let cases = inventory["cases"]
        .as_array_mut()
        .ok_or_else(|| Error::new("suite cases must be an array"))?;
    let at = cases
        .iter()
        .position(|case| {
            case["id"]
                .as_str()
                .is_some_and(|id| id.starts_with(&prefix))
        })
        .unwrap_or(cases.len());
    cases.retain(|case| {
        !case["id"]
            .as_str()
            .is_some_and(|id| id.starts_with(&prefix))
    });
    let replacement = export
        .cases
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    cases.splice(at..at, replacement);
    let bases = inventory["bases"]
        .as_object_mut()
        .ok_or_else(|| Error::new("suite bases must be an object"))?;
    for (id, base) in export.bases {
        if id != "live" {
            bases.insert(id, serde_json::to_value(base)?);
        }
    }
    Ok(())
}

pub(crate) fn run(context: &Context, family: &str) -> Result<(), Error> {
    if family == "spine" {
        println!(
            "{}",
            spine::run(
                context,
                Path::new("new-book-plans/constitution.nibli"),
                Path::new("new-book-plans/3-spine.md"),
            )?
        );
        return Ok(());
    }
    if !matches!(family, "state-form" | "obligations") {
        return Err(Error::usage(
            "usage: ./generate.sh state-form|obligations|spine",
        ));
    }
    let mut inventory: serde_json::Value =
        serde_json::from_str(&context.read("tests/pins/suites.json")?)?;
    let mut export = Export::new();
    match family {
        "state-form" => state_form::generate(context, &mut export)?,
        "obligations" => obligations::generate(context, &mut export)?,
        _ => unreachable!(),
    }
    let count = export.cases.len();
    merge_family(&mut inventory, family, export)?;
    let text = format!("{}\n", serde_json::to_string_pretty(&inventory)?);
    std::fs::write(context.path("tests/pins/suites.json"), text)?;
    println!("generated {family} rules, pins, and {count} executable cases");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn isolated_authoring(family: &str) -> (tempfile::TempDir, Context, String) {
        let live = Context::discover().expect("repository");
        let directory = tempfile::tempdir().expect("temporary authoring root");
        let context = Context::from_test_root(directory.path().to_owned());
        std::fs::create_dir_all(context.path("new-book-plans/counterfactual")).unwrap();
        std::fs::create_dir_all(context.path("tests/pins")).unwrap();
        for path in [
            "new-book-plans/constitution.nibli",
            &format!("new-book-plans/{family}-source.json"),
        ] {
            std::fs::copy(live.path(path), context.path(path)).unwrap();
        }
        let inventory = serde_json::json!({
            "bases": {"live": {"path": "new-book-plans/constitution.nibli"}},
            "cases": [{"id":"unrelated/example", "base":"live", "pins":["unrelated.pins.nibli"], "allow_shell":true}]
        });
        std::fs::write(
            context.path("tests/pins/suites.json"),
            inventory.to_string(),
        )
        .unwrap();
        let before = context.read("new-book-plans/constitution.nibli").unwrap();
        (directory, context, before)
    }

    fn assert_outside_region_unchanged(before: &str, after: &str, begin: &str, end: &str) {
        assert_eq!(
            before.split_once(begin).unwrap().0,
            after.split_once(begin).unwrap().0
        );
        assert_eq!(
            before.split_once(end).unwrap().1,
            after.split_once(end).unwrap().1
        );
    }

    fn assert_fixture_statements_terminated(context: &Context) {
        let inventory: serde_json::Value =
            serde_json::from_str(&context.read("tests/pins/suites.json").unwrap()).unwrap();
        for case in inventory["cases"].as_array().unwrap() {
            for path in case["fixtures"].as_array().into_iter().flatten() {
                let path = path.as_str().unwrap();
                for (index, line) in context.read(path).unwrap().lines().enumerate() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        assert!(
                            line.ends_with('.'),
                            "{path}:{}: unterminated fixture statement",
                            index + 1
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn state_form_generation_preserves_unrelated_rules_and_suite_settings() {
        let (_directory, context, before) = isolated_authoring("state-form");
        run(&context, "state-form")
            .expect("generate the live authoring input without digest prerequisites");
        assert_fixture_statements_terminated(&context);
        let after = context.read("new-book-plans/constitution.nibli").unwrap();
        assert_outside_region_unchanged(
            &before,
            &after,
            "# <STATE-FORM-RULES-BEGIN>",
            "# <STATE-FORM-RULES-END>",
        );
        let inventory: serde_json::Value =
            serde_json::from_str(&context.read("tests/pins/suites.json").unwrap()).unwrap();
        assert_eq!(inventory["cases"][0]["allow_shell"], true);
        assert!(
            inventory["cases"]
                .as_array()
                .unwrap()
                .iter()
                .any(|case| case["id"] == "state-form/main-01")
        );
        let first = context
            .read("new-book-plans/state-form.pins.nibli")
            .unwrap();
        run(&context, "state-form").expect("repeat generation");
        assert_eq!(
            context
                .read("new-book-plans/state-form.pins.nibli")
                .unwrap(),
            first
        );
        assert_eq!(
            context.read("new-book-plans/constitution.nibli").unwrap(),
            after
        );
    }

    #[test]
    fn obligations_generation_needs_no_assurance_ledger_and_is_repeatable() {
        let (_directory, context, before) = isolated_authoring("obligations");
        assert!(
            !context
                .path("new-book-plans/full-society-ledger.json")
                .exists()
        );
        run(&context, "obligations").expect("generate from the small semantic input");
        assert_fixture_statements_terminated(&context);
        let after = context.read("new-book-plans/constitution.nibli").unwrap();
        assert_outside_region_unchanged(
            &before,
            &after,
            "# <OBLIGATIONS-RULES-BEGIN>",
            "# <OBLIGATIONS-RULES-END>",
        );
        let inventory = context.read("tests/pins/suites.json").unwrap();
        run(&context, "obligations").expect("repeat generation");
        assert_eq!(context.read("tests/pins/suites.json").unwrap(), inventory);
        assert_eq!(
            context.read("new-book-plans/constitution.nibli").unwrap(),
            after
        );
    }
}
