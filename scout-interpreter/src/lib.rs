use std::collections::HashSet;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use env::EnvPointer;
use fantoccini::Locator;
use futures::lock::Mutex;
use futures::{future::BoxFuture, FutureExt};
use object::{obj_map_to_json, Object};
use scout_lexer::{Lexer, TokenKind};
use scout_parser::ast::{
    Block, CrawlLiteral, ExprKind, Identifier, IfElseLiteral, NodeKind, Program, StmtKind,
};
use scout_parser::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::{builtin::BuiltinKind, env::Env};

pub mod builtin;
pub mod env;
pub mod object;

pub type EvalResult = Result<Arc<Object>, EvalError>;
pub type ScrapeResultsPtr = Arc<Mutex<ScrapeResults>>;

const MAX_DEPTH: usize = 10;

/// Evaluates the block and early returns if the stmt evaluates
/// to a Return
macro_rules! check_return_eval {
    ($block:expr, $crawler:expr, $env:expr, $results:expr) => {
        if let $crate::object::Object::Return(ret) =
            &*eval_block($block, $crawler, $env, $results).await?
        {
            return Ok(ret.clone());
        }
    };
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ScrapeResults {
    results: Map<String, Value>,
}

impl ScrapeResults {
    pub fn add_result(&mut self, res: Map<String, Value>, url: &str) {
        match self.results.get_mut(url) {
            None => {
                self.results.insert(url.to_owned(), vec![res].into());
            }
            Some(Value::Array(v)) => {
                v.push(Value::from(res));
            }
            // This should never happen since `add_results` is the only way to
            // insert to the map.
            _ => panic!("results was not a vec type"),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

// TODO add parameters for better debugging.
#[derive(Debug)]
pub enum EvalError {
    TypeMismatch,
    InvalidUsage(String),
    InvalidFnParams,
    InvalidExpr,
    InvalidUrl,
    InvalidImport,
    InvalidIndex,
    NonFunction,
    UnknownIdent,
    UnknownPrefixOp,
    UnknownInfixOp,
    UncaughtException,
    URLParseError(String),
    DuplicateDeclare,
    NonIterable,
    ScreenshotError,
    BrowserError(fantoccini::error::CmdError),
    OSError,
}

pub async fn eval(
    node: NodeKind,
    crawler: &fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> EvalResult {
    use NodeKind::*;
    match node {
        Program(p) => eval_program(p, crawler, env.clone(), results.clone()).await,
        Stmt(s) => eval_statement(&s, crawler, env.clone(), results.clone()).await,
        Expr(e) => eval_expression(&e, crawler, env.clone(), results.clone()).await,
    }
}

async fn eval_program(
    prgm: Program,
    crawler: &fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> EvalResult {
    let block = Block::new(prgm.stmts);
    eval_block(&block, crawler, env.clone(), results.clone()).await
}

fn eval_statement<'a>(
    stmt: &'a StmtKind,
    crawler: &'a fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> BoxFuture<'a, EvalResult> {
    async move {
        match stmt {
            StmtKind::Goto(expr) => {
                if let Object::Str(url) =
                    &*eval_expression(expr, crawler, env.clone(), results.clone()).await?
                {
                    if crawler.goto(url.as_str()).await.is_err() {
                        return Err(EvalError::InvalidUrl);
                    };
                } else {
                    return Err(EvalError::InvalidFnParams);
                }

                wait_for_goto_ready().await;

                Ok(Arc::new(Object::Null))
            }
            StmtKind::TryCatch(try_block, catch_block) => {
                match eval_block(try_block, crawler, env.clone(), results.clone())
                    .await
                    .as_deref()
                {
                    Ok(Object::Return(ret)) => return Ok(ret.clone()),
                    // if it was successful but not a return, do nothing
                    Ok(_) => {}
                    Err(_) if catch_block.is_some() => {
                        let block = catch_block.as_ref().unwrap();
                        check_return_eval!(block, crawler, env.clone(), results.clone());
                    }
                    Err(_) => return Err(EvalError::UncaughtException),
                };

                Ok(Arc::new(Object::Null))
            }
            StmtKind::Scrape(defs) => {
                let mut res = HashMap::new();
                for (id, def) in &defs.pairs {
                    let val = eval_expression(def, crawler, env.clone(), results.clone()).await?;
                    res.insert(id.clone(), val);
                }
                results.lock().await.add_result(
                    obj_map_to_json(&res),
                    crawler.current_url().await.unwrap().as_str(),
                );
                Ok(Arc::new(Object::Null))
            }
            StmtKind::Expr(expr) => {
                eval_expression(expr, crawler, env.clone(), results.clone()).await
            }
            StmtKind::ForLoop(floop) => {
                let items =
                    eval_expression(&floop.iterable, crawler, env.clone(), results.clone()).await?;
                if let Some(iterable) = items.into_iterable() {
                    for obj in iterable.into_iter().collect::<Vec<Arc<Object>>>() {
                        let mut scope = Env::default();
                        scope.add_outer(env.clone()).await;
                        scope.set(&floop.ident, obj.clone()).await;
                        check_return_eval!(
                            &floop.block,
                            crawler,
                            Arc::new(Mutex::new(scope)),
                            results.clone()
                        );
                    }
                    Ok(Arc::new(Object::Null))
                } else {
                    Err(EvalError::NonIterable)
                }
            }
            StmtKind::Assign(ident, expr) => {
                let val = eval_expression(expr, crawler, env.clone(), results.clone()).await?;
                env.lock().await.set(ident, val).await;
                Ok(Arc::new(Object::Null))
            }
            StmtKind::Screenshot(path) => {
                let png = crawler.screenshot().await?;
                let img = image::io::Reader::new(std::io::Cursor::new(png))
                    .with_guessed_format()
                    .map_err(|_| EvalError::ScreenshotError)?
                    .decode()?;
                img.save(path)?;

                Ok(Arc::new(Object::Null))
            }
            StmtKind::IfElse(IfElseLiteral {
                if_lit,
                elifs,
                else_lit,
            }) => {
                let truth_check =
                    eval_expression(&if_lit.cond, crawler, env.clone(), results.clone()).await?;
                if truth_check.is_truthy() {
                    check_return_eval!(&if_lit.block, crawler, env.clone(), results.clone());
                } else {
                    for elif in elifs {
                        if eval_expression(&elif.cond, crawler, env.clone(), results.clone())
                            .await?
                            .is_truthy()
                        {
                            check_return_eval!(&elif.block, crawler, env.clone(), results.clone());
                            return Ok(Arc::new(Object::Null));
                        }
                    }

                    if let Some(lit) = else_lit {
                        check_return_eval!(&lit.block, crawler, env.clone(), results.clone());
                    }
                }

                Ok(Arc::new(Object::Null))
            }
            StmtKind::Func(def) => {
                let lit = Object::Fn(def.params.clone(), def.body.clone());
                env.lock().await.set(&def.ident, Arc::new(lit)).await;
                Ok(Arc::new(Object::Null))
            }
            StmtKind::Return(rv) => match rv {
                None => Ok(Arc::new(Object::Null)),
                Some(expr) => eval_expression(expr, crawler, env.clone(), results.clone()).await,
            },
            StmtKind::Use(import) => {
                println!("{import:?}");
                let file = eval_expression(import, crawler, env.clone(), results.clone()).await?;
                match &*file {
                    Object::Str(f_str) => {
                        let curr_dir = std::env::current_dir().map_err(|_| EvalError::OSError)?;
                        let f_path = std::path::Path::new(f_str);
                        let mut path = curr_dir.join(f_path);
                        path.set_extension("sct");
                        match path.to_str() {
                            Some(path) => {
                                let content =
                                    fs::read_to_string(path).map_err(|_| EvalError::OSError)?;
                                let lex = Lexer::new(&content);
                                let mut parser = Parser::new(lex);
                                match parser.parse_program() {
                                    Ok(prgm) => {
                                        let module_env = Arc::new(Mutex::new(Env::default()));
                                        let _ = eval(
                                            NodeKind::Program(prgm),
                                            crawler,
                                            module_env.clone(),
                                            results.clone(),
                                        )
                                        .await?;
                                        Ok(Arc::new(Object::Module(module_env)))
                                    }
                                    Err(_) => Err(EvalError::InvalidImport),
                                }
                            }
                            None => Err(EvalError::InvalidImport),
                        }
                    }
                    _ => Err(EvalError::InvalidImport),
                }
            }
            StmtKind::Crawl(lit) => {
                let mut visited = HashSet::new();

                eval_crawl(lit, crawler, env, results, &mut visited, 1).await?;

                Ok(Arc::new(Object::Null))
            }
        }
    }
    .boxed()
}

fn eval_crawl<'a>(
    lit: &'a CrawlLiteral,
    crawler: &'a fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
    visited: &'a mut HashSet<String>,
    depth: usize,
) -> BoxFuture<'a, Result<(), EvalError>> {
    async move {
        let start = crawler.window().await?;
        match crawler.find_all(Locator::Css("a[href]")).await {
            Ok(elems) => {
                for elem in elems.iter() {
                    if let Ok(Some(link_str)) = elem.attr("href").await {
                        let curr_url = crawler.current_url().await?;
                        let link = match url::Url::parse(&link_str) {
                            Ok(l) => Ok(l.to_string()),
                            Err(url::ParseError::RelativeUrlWithoutBase) => Ok(curr_url
                                .join(&link_str)
                                .map_err(|_| EvalError::InvalidUrl)?
                                .to_string()),
                            Err(_) => Err(EvalError::InvalidUrl),
                        }?;

                        let mut scope = Env::default();
                        scope.add_outer(env.clone()).await;

                        if let Some(bindings) = &lit.bindings {
                            scope
                                .set(&bindings.link, Arc::new(Object::Str(link.clone())))
                                .await;
                            scope
                                .set(&bindings.depth, Arc::new(Object::Number(depth as f64)))
                                .await;
                        }

                        let new_env = Arc::new(Mutex::new(scope));

                        let mut truth_check = true;
                        if let Some(expr) = &lit.filter {
                            let obj =
                                eval_expression(expr, crawler, new_env.clone(), results.clone())
                                    .await?;
                            truth_check = obj.is_truthy();
                        }
                        if !visited.contains(&link) && truth_check {
                            let new_tab = crawler.new_window(true).await?;
                            crawler.switch_to_window(new_tab.handle).await?;
                            let _ = crawler.goto(&link).await;

                            // Add both the starting url and resolved url to the visited.
                            visited.insert(link);
                            visited.insert(crawler.current_url().await?.to_string());

                            eval_block(&lit.body, crawler, new_env.clone(), results.clone())
                                .await?;

                            if depth < MAX_DEPTH {
                                eval_crawl(
                                    lit,
                                    crawler,
                                    env.clone(),
                                    results.clone(),
                                    visited,
                                    depth + 1,
                                )
                                .await?;
                            }

                            crawler.switch_to_window(start.clone()).await?;
                        }
                    }
                }
            }
            Err(e) => return Err(EvalError::BrowserError(e)),
        };

        Ok(())
    }
    .boxed()
}

async fn wait_for_goto_ready() {
    // @TODO: Need a better way to determine that a page is "done"
    sleep(Duration::from_millis(50));
}

async fn eval_block(
    block: &Block,
    crawler: &fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> EvalResult {
    let mut res = Arc::new(Object::Null);
    for stmt in &block.stmts {
        match stmt {
            StmtKind::Return(rv) => {
                return match rv {
                    None => Ok(Arc::new(Object::Return(Arc::new(Object::Null)))),
                    Some(expr) => Ok(Arc::new(Object::Return(
                        eval_expression(expr, crawler, env.clone(), results.clone()).await?,
                    ))),
                }
            }
            _ => {
                res = eval_statement(stmt, crawler, env.clone(), results.clone())
                    .await?
                    .clone()
            }
        }
    }
    Ok(res)
}

fn apply_call<'a>(
    ident: &'a Identifier,
    params: &'a [ExprKind],
    crawler: &'a fantoccini::Client,
    prev: Option<Arc<Object>>,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> BoxFuture<'a, EvalResult> {
    async move {
        // Evaluate the provided fn inputs
        let mut obj_params = Vec::new();
        for param in params.iter() {
            let expr = eval_expression(param, crawler, env.clone(), results.clone()).await?;
            obj_params.push(expr);
        }

        // Insert into the beginning of fn inputs if the
        // fn is being piped
        if let Some(obj) = prev {
            obj_params.insert(0, obj);
        }

        // Set var before match to avoid deadlock on env
        let env_res = env.lock().await.get(ident).await;
        match env_res {
            // This is a user defined function
            Some(obj) => match &*obj {
                // Only fn's are callable
                Object::Fn(fn_params, block) => {
                    // Create the scope that will be used within the fn body
                    let mut scope = Env::default();
                    scope.add_outer(env.clone()).await;
                    for (i, fn_param) in fn_params.iter().enumerate() {
                        let id = &fn_param.ident;
                        // check if the fn was provided this param or if
                        // we should use a default
                        match obj_params.get(i) {
                            // Fn param was provided
                            Some(provided) => {
                                scope.set(id, provided.clone()).await;
                            }
                            // Fn param was not provided, check for defaults
                            None => match &fn_param.default {
                                Some(def) => {
                                    let obj_def =
                                        eval_expression(def, crawler, env.clone(), results.clone())
                                            .await?;
                                    scope.set(id, obj_def).await;
                                }
                                None => {
                                    return Err(EvalError::InvalidFnParams);
                                }
                            },
                        }
                    }

                    let ev =
                        eval_block(block, crawler, Arc::new(Mutex::new(scope)), results.clone())
                            .await?;
                    match &*ev {
                        Object::Return(ret) => Ok(ret.clone()),
                        _ => Ok(ev),
                    }
                }
                _ => Err(EvalError::InvalidExpr),
            },
            // Not user defined, check if its a builtin
            None => match BuiltinKind::is_from(&ident.name) {
                Some(builtin) => builtin.apply(crawler, results.clone(), obj_params).await,
                None => Err(EvalError::UnknownIdent),
            },
        }
    }
    .boxed()
}

async fn apply_debug_border(crawler: &fantoccini::Client, selector: &str) {
    let js = r#"
    const [selector] = arguments;

    document.querySelector(selector).style.boxShadow = "0 0 0 5px red";
    document.querySelector(selector).style.outline = "dashed 5px yellow";
    "#;
    let _ = crawler.execute(js, vec![json!(selector)]).await;
}

async fn apply_debug_border_all(crawler: &fantoccini::Client, selector: &str) {
    let js = r#"
    const [selector] = arguments;

    document.querySelectorAll(selector).forEach(elem => elem.style.boxShadow = "0 0 0 5px red");
    document.querySelectorAll(selector).forEach(elem => elem.style.outline = "dashed 5px yellow");
    "#;
    let _ = crawler.execute(js, vec![json!(selector)]).await;
}

fn eval_expression<'a>(
    expr: &'a ExprKind,
    crawler: &'a fantoccini::Client,
    env: EnvPointer,
    results: ScrapeResultsPtr,
) -> BoxFuture<'a, EvalResult> {
    async move {
        match expr {
            ExprKind::Select(selector, scope) => match scope {
                Some(ident) => match env.lock().await.get(ident).await.as_deref() {
                    Some(Object::Node(elem)) => match elem.find(Locator::Css(selector)).await {
                        Ok(node) => {
                            // @TODO fix - applies borders outside scope
                            apply_debug_border(crawler, selector).await;
                            Ok(Arc::new(Object::Node(node)))
                        }
                        Err(_) => Ok(Arc::new(Object::Null)),
                    },
                    Some(_) => Err(EvalError::InvalidUsage("Cannot select non-node".into())),
                    None => Err(EvalError::UnknownIdent),
                },
                None => match crawler.find(Locator::Css(selector)).await {
                    Ok(node) => {
                        apply_debug_border(crawler, selector).await;
                        Ok(Arc::new(Object::Node(node)))
                    }
                    Err(_) => Ok(Arc::new(Object::Null)),
                },
            },
            ExprKind::SelectAll(selector, scope) => match scope {
                Some(ident) => match env.lock().await.get(ident).await.as_deref() {
                    Some(Object::Node(elem)) => match elem.find_all(Locator::Css(selector)).await {
                        Ok(nodes) => {
                            // @TODO fix - applies borders outside scope
                            apply_debug_border_all(crawler, selector).await;
                            let elems = nodes
                                .iter()
                                .map(|e| Arc::new(Object::Node(e.clone())))
                                .collect();
                            Ok(Arc::new(Object::List(elems)))
                        }
                        Err(_) => Ok(Arc::new(Object::Null)),
                    },
                    Some(_) => Err(EvalError::InvalidUsage("cannot select non-node".into())),
                    None => Err(EvalError::UnknownIdent),
                },
                None => match crawler.find_all(Locator::Css(selector)).await {
                    Ok(nodes) => {
                        apply_debug_border_all(crawler, selector).await;
                        let elems = nodes
                            .iter()
                            .map(|e| Arc::new(Object::Node(e.clone())))
                            .collect();
                        Ok(Arc::new(Object::List(elems)))
                    }
                    Err(_) => Ok(Arc::new(Object::Null)),
                },
            },
            ExprKind::Str(s) => Ok(Arc::new(Object::Str(s.to_owned()))),
            ExprKind::Number(n) => Ok(Arc::new(Object::Number(*n))),
            ExprKind::Call(ident, params) => {
                apply_call(ident, params, crawler, None, env.clone(), results.clone()).await
            }
            ExprKind::Ident(ident) => match env.lock().await.get(ident).await {
                Some(obj) => Ok(obj.clone()),
                None => Err(EvalError::UnknownIdent),
            },
            ExprKind::Chain(exprs) => {
                let mut prev: Option<Arc<Object>> = None;
                for expr in exprs {
                    let eval = match expr {
                        ExprKind::Call(ident, params) => {
                            apply_call(ident, params, crawler, prev, env.clone(), results.clone())
                                .await?
                        }
                        _ => eval_expression(expr, crawler, env.clone(), results.clone()).await?,
                    };
                    prev = Some(eval);
                }
                Ok(prev.unwrap())
            }
            ExprKind::Infix(lhs, op, rhs) => {
                let l_obj = eval_expression(lhs, crawler, env.clone(), results.clone()).await?;
                let r_obj = eval_expression(rhs, crawler, env.clone(), results.clone()).await?;
                let res = eval_infix(l_obj.clone(), op, r_obj.clone())?;
                Ok(res)
            }
            ExprKind::Boolean(val) => Ok(Arc::new(Object::Boolean(*val))),
            ExprKind::Null => Ok(Arc::new(Object::Null)),
            ExprKind::List(vec) => {
                let mut list_content = Vec::new();
                for expr in vec {
                    let obj = eval_expression(expr, crawler, env.clone(), results.clone()).await?;
                    list_content.push(obj);
                }

                Ok(Arc::new(Object::List(list_content)))
            }
            ExprKind::Prefix(rhs, op) => {
                let r_obj = eval_expression(rhs, crawler, env.clone(), results.clone()).await?;
                let res = eval_prefix(r_obj, op)?;
                Ok(res)
            }
        }
    }
    .boxed()
}

fn eval_infix(lhs: Arc<Object>, op: &TokenKind, rhs: Arc<Object>) -> EvalResult {
    match op {
        TokenKind::EQ => Ok(Arc::new(Object::Boolean(lhs == rhs))),
        TokenKind::NEQ => Ok(Arc::new(Object::Boolean(lhs != rhs))),
        TokenKind::Plus => eval_plus_op(lhs, rhs),
        TokenKind::Minus => eval_minus_op(lhs, rhs),
        TokenKind::Asterisk => eval_asterisk_op(lhs, rhs),
        TokenKind::Slash => eval_slash_op(lhs, rhs),
        TokenKind::LBracket => eval_index(lhs, rhs),
        TokenKind::GT => eval_gt_op(lhs, rhs),
        TokenKind::LT => eval_lt_op(lhs, rhs),
        TokenKind::GTE => eval_gte_op(lhs, rhs),
        TokenKind::LTE => eval_lte_op(lhs, rhs),
        TokenKind::And => Ok(Arc::new(Object::Boolean(
            lhs.is_truthy() && rhs.is_truthy(),
        ))),
        TokenKind::Or => Ok(Arc::new(Object::Boolean(
            lhs.is_truthy() || rhs.is_truthy(),
        ))),
        TokenKind::DbColon => eval_dbcolon_op(lhs, rhs),
        _ => Err(EvalError::UnknownInfixOp),
    }
}

fn eval_dbcolon_op(lhs: Arc<Object>, rhs: Arc<Object>) -> EvalResult {
    println!("lhs {lhs:?}, rhs {rhs:?}");
    match (&*lhs, &*rhs) {
        _ => Err(EvalError::UnknownInfixOp),
    }
}

fn eval_prefix(rhs: Arc<Object>, op: &TokenKind) -> EvalResult {
    match (&*rhs, op) {
        (_, TokenKind::Bang) => {
            let truth = !rhs.is_truthy();
            Ok(Arc::new(Object::Boolean(truth)))
        }
        _ => Err(EvalError::UnknownPrefixOp),
    }
}

fn eval_index(lhs: Arc<Object>, idx: Arc<Object>) -> EvalResult {
    match (&*lhs, &*idx) {
        (Object::List(a), Object::Number(b)) => Ok(a[*b as usize].clone()),
        _ => Err(EvalError::InvalidIndex),
    }
}

fn eval_gt_op(lhs: Arc<Object>, rhs: Arc<Object>) -> EvalResult {
    match (&*lhs, &*rhs) {
        (Object::Number(a), Object::Number(b)) => Ok(Arc::new(Object::Boolean(a > b))),
        _ => Err(EvalError::UnknownInfixOp),
    }
}

fn eval_gte_op(lhs: Arc<Object>, rhs: Arc<Object>) -> EvalResult {
    match (&*lhs, &*rhs) {
        (Object::Number(a), Object::Number(b)) => Ok(Arc::new(Object::Boolean(a >= b))),
        _ => Err(EvalError::UnknownInfixOp),
    }
}

fn eval_lt_op(lhs: Arc<Object>, rhs: Arc<Object>) -> EvalResult {
    match (&*lhs, &*rhs) {
        (Object::Number(a), Object::Number(b)) => Ok(Arc::new(Object::Boolean(a < b))),
        _ => Err(EvalError::UnknownInfixOp),
    }
}

fn eval_lte_op(lhs: Arc<Object>, rhs: Arc<Object>) -> EvalResult {
    match (&*lhs, &*rhs) {
        (Object::Number(a), Object::Number(b)) => Ok(Arc::new(Object::Boolean(a <= b))),
        _ => Err(EvalError::UnknownInfixOp),
    }
}

fn eval_plus_op(lhs: Arc<Object>, rhs: Arc<Object>) -> EvalResult {
    match (&*lhs, &*rhs) {
        (Object::Str(a), Object::Str(b)) => {
            let res = format!("{a}{b}");
            Ok(Arc::new(Object::Str(res)))
        }
        (Object::Number(a), Object::Number(b)) => Ok(Arc::new(Object::Number(a + b))),
        _ => Err(EvalError::UnknownInfixOp),
    }
}

fn eval_minus_op(lhs: Arc<Object>, rhs: Arc<Object>) -> EvalResult {
    match (&*lhs, &*rhs) {
        (Object::Number(a), Object::Number(b)) => Ok(Arc::new(Object::Number(a - b))),
        _ => Err(EvalError::UnknownInfixOp),
    }
}

fn eval_asterisk_op(lhs: Arc<Object>, rhs: Arc<Object>) -> EvalResult {
    match (&*lhs, &*rhs) {
        (Object::Number(a), Object::Number(b)) => Ok(Arc::new(Object::Number(a * b))),
        _ => Err(EvalError::UnknownInfixOp),
    }
}

fn eval_slash_op(lhs: Arc<Object>, rhs: Arc<Object>) -> EvalResult {
    match (&*lhs, &*rhs) {
        (Object::Number(a), Object::Number(b)) => Ok(Arc::new(Object::Number(a / b))),
        _ => Err(EvalError::UnknownInfixOp),
    }
}

impl From<fantoccini::error::CmdError> for EvalError {
    fn from(e: fantoccini::error::CmdError) -> Self {
        Self::BrowserError(e)
    }
}

impl From<image::ImageError> for EvalError {
    fn from(_: image::ImageError) -> Self {
        Self::ScreenshotError
    }
}
