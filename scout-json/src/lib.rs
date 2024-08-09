use scout_parser::ast::{CallLiteral, ExprKind, Identifier, Program, StmtKind};
use serde::Deserialize;

/// ScoutJson is a JSON representation of a subset of the Scout AST.
/// It is meant to model after the Google Chrome Recorder API.
#[derive(Debug, Deserialize)]
pub struct ScoutJSON {
    steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Step {
    SetViewport { width: u32, height: u32 },
    Navigate { url: String },
    Click { selectors: Vec<Vec<String>> },
}

impl ScoutJSON {
    pub fn to_ast(&self) -> Program {
        let mut stmts = Vec::new();
        for step in &self.steps {
            stmts.push(step.to_stmt());
        }

        Program { stmts }
    }
}

impl Step {
    pub fn to_stmt(&self) -> StmtKind {
        use Step::*;
        match self {
            SetViewport { width, height } => {
                let lit = CallLiteral {
                    ident: Identifier::new("setViewport".to_string()),
                    args: vec![
                        ExprKind::Number(*width as f64),
                        ExprKind::Number(*height as f64),
                    ],
                    kwargs: Vec::new(),
                };
                StmtKind::Expr(ExprKind::Call(lit))
            }
            Navigate { url } => StmtKind::Goto(ExprKind::Str(url.clone())),
            Click { selectors } => {
                // By default, chrome outputs an arry and the length depends upon what
                // outputs are set in the recording. We will assume only CSS is set as
                // the others are not usable by scout yet.
                // The css value is an array of length 1, ex:
                //
                // "selectors": [
                //     [
                //         "#question-summary-78853169 h3 > a"
                //     ]
                // ]
                let elem = ExprKind::Select(selectors[0][0].clone(), None);
                let lit = CallLiteral {
                    ident: Identifier::new("click".to_string()),
                    args: vec![elem],
                    kwargs: Vec::new(),
                };
                StmtKind::Expr(ExprKind::Call(lit))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(
        r#"{
            "type": "navigate",
            "url": "https://stackoverflow.com/",
            "assertedEvents": [
                {
                    "type": "navigation",
                    "url": "https://stackoverflow.com/",
                    "title": ""
                }
            ]
        }"#,
        StmtKind::Goto(ExprKind::Str("https://stackoverflow.com/".to_string()));
        "navigate step"
    )]
    #[test_case(
        r##"{
            "type": "click",
            "target": "main",
            "selectors": [
                [
                    "#question-summary-78853169 h3 > a"
                ]
            ],
            "offsetY": 2.875,
            "offsetX": 183,
            "assertedEvents": [
                {
                    "type": "navigation",
                    "url": "https://stackoverflow.com/questions/78853169/how-can-i-pass-variables-to-svelte-through-csv",
                    "title": "typescript - How can I pass variables to svelte through CSV - Stack Overflow"
                }
            ]
        }"##,
        StmtKind::Expr(ExprKind::Call(CallLiteral {
            ident: Identifier::new("click".to_string()),
            args: vec![ExprKind::Select("#question-summary-78853169 h3 > a".to_string(), None)],
            kwargs: Vec::new(),
        }));
        "click step"
    )]
    #[test_case(
        r#"{
            "type": "setViewport",
            "width": 1365,
            "height": 945,
            "deviceScaleFactor": 1,
            "isMobile": false,
            "hasTouch": false,
            "isLandscape": false
        }"#,
        StmtKind::Expr(ExprKind::Call(CallLiteral {
            ident: Identifier::new("setViewport".to_string()),
            args: vec![
                ExprKind::Number(1365.),
                ExprKind::Number(945.),
            ],
            kwargs: Vec::new(),
        }));
        "setViewport step"
    )]
    fn parse_step_json(input: &str, exp: StmtKind) {
        assert_eq!(exp, serde_json::from_str::<Step>(input).unwrap().to_stmt())
    }
}
