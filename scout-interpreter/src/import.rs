use std::{
    env,
    path::{Path, PathBuf},
};

use scout_lexer::TokenKind;
use scout_parser::ast::{ExprKind, Identifier};

use crate::EvalError;

#[derive(Debug)]
pub struct ResolvedMod {
    pub ident: Identifier,
    pub filepath: String,
}

pub fn resolve_module(module: &ExprKind) -> Result<ResolvedMod, EvalError> {
    let ident = match module {
        ExprKind::Ident(ident) => Ok(ident.clone()),
        ExprKind::Infix(_, _, rhs) => match rhs.as_ref() {
            ExprKind::Ident(ident) => Ok(ident.clone()),
            _ => Err(EvalError::InvalidImport),
        },
        _ => Err(EvalError::InvalidImport),
    }?;
    let buf = resolve_module_file(module)?;
    let filepath = convert_path_buf(buf)?;
    Ok(ResolvedMod { filepath, ident })
}

fn resolve_std_file(ident: &Identifier) -> Result<PathBuf, EvalError> {
    if *ident == Identifier::new("std".into()) {
        let scout_dir = match env::var("SCOUT_PATH") {
            Ok(s) => Ok(Path::new(&s).to_path_buf()),
            Err(_) => match env::var("HOME") {
                Ok(s) => Ok(Path::new(&s).join("scout-lang")),
                Err(_) => Err(EvalError::OSError),
            },
        }?;
        // let root = env::var("SCOUT_PATH")
        //     .or(env::var("HOME"))
        //     .map_err(|_| EvalError::OSError)?;
        // let home = env::var("HOME").map_err(|_| EvalError::OSError)?;
        let path = scout_dir.join("scout-lib").to_owned();
        Ok(path)
    } else {
        Ok(Path::new(&ident.name).to_owned())
    }
}

fn convert_path_buf(buf: PathBuf) -> Result<String, EvalError> {
    let res = buf.to_str().ok_or(EvalError::InvalidImport)?.to_owned();
    Ok(res)
}

fn resolve_module_file(module: &ExprKind) -> Result<PathBuf, EvalError> {
    match module {
        ExprKind::Ident(ident) => resolve_std_file(ident),
        ExprKind::Infix(lhs, TokenKind::DbColon, rhs) => match (lhs.as_ref(), rhs.as_ref()) {
            (ExprKind::Ident(base), ExprKind::Ident(file)) => {
                let buf = resolve_std_file(base)?.join(&file.name);
                Ok(buf)
            }
            (l @ ExprKind::Infix(_, TokenKind::DbColon, _), ExprKind::Ident(file)) => {
                let base = resolve_module_file(l)?;
                let buf = base.join(&file.name);
                Ok(buf)
            }
            _ => Err(EvalError::InvalidImport),
        },
        _ => Err(EvalError::InvalidImport),
    }
}
