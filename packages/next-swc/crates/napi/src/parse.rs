use crate::util::{deserialize_json, MapErr};
use anyhow::Context as _;
use napi::bindgen_prelude::*;
use std::sync::Arc;
use swc::{config::ParseOptions, try_with_handler};
use swc_common::{comments::Comments, errors::ColorConfig, FileName, FilePathMapping, SourceMap};

pub struct ParseTask {
    pub filename: FileName,
    pub src: String,
    pub options: String,
}

#[napi]
impl Task for ParseTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let c = swc::Compiler::new(Arc::new(SourceMap::new(FilePathMapping::empty())));

        let options: ParseOptions = deserialize_json(&self.options).convert_err()?;
        let comments = c.comments().clone();
        let comments: Option<&dyn Comments> = if options.comments {
            Some(&comments)
        } else {
            None
        };
        let fm =
            c.cm.new_source_file(self.filename.clone(), self.src.clone());
        let program = try_with_handler(
            c.cm.clone(),
            swc::HandlerOpts {
                color: ColorConfig::Never,
                skip_filename: false,
            },
            |handler| {
                c.parse_js(
                    fm,
                    handler,
                    options.target,
                    options.syntax,
                    options.is_module,
                    comments,
                )
            },
        )
        .convert_err()?;

        let ast_json = serde_json::to_string(&program)
            .context("failed to serialize Program")
            .convert_err()?;

        Ok(ast_json)
    }

    fn resolve(&mut self, _: Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

#[napi]
pub fn parse(
    src: String,
    options: Buffer,
    filename: Option<String>,
) -> napi::Result<AsyncTask<ParseTask>> {
    let options = serde_json::from_slice(&options)?;
    let filename = if let Some(value) = filename {
        FileName::Real(value.into())
    } else {
        FileName::Anon
    };

    Ok(AsyncTask::new(ParseTask {
        filename,
        src,
        options,
    }))
}
