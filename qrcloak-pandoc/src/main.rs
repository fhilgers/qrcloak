use std::{
    io::Write,
    ops::Range,
    process::{Command, Stdio},
    str::{from_utf8, FromStr},
};

use age::x25519::Recipient;
use miette::{miette, Diagnostic, IntoDiagnostic, LabeledSpan, NamedSource, Result, SourceSpan};
use pandoc_ast::Pandoc;
use qrcloak_core::payload::AgeKeyOptions;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Diagnostic, Debug, Error)]
enum FilterError {
    #[error("missing field `{argument_name}`")]
    #[diagnostic()]
    MissingArgument {
        argument_name: String,
        #[source_code]
        src: NamedSource<String>,

        #[label]
        block_span: SourceSpan,

        #[label("here")]
        here_span: SourceSpan,

        #[help]
        advice: String,
    },

    #[error("invalid value for field `{argument_name}`")]
    #[diagnostic()]
    InvalidArgument {
        argument_name: String,

        #[source_code]
        src: NamedSource<String>,

        #[label]
        block_span: SourceSpan,

        #[label(collection)]
        spans: Vec<LabeledSpan>,

        #[help]
        advice: String,
    },
}

#[derive(Debug)]
pub struct Code {
    attrs: Attrs,
    data: String,
}

impl From<&((String, Vec<String>, Vec<(String, String)>), String)> for Code {
    fn from(value: &((String, Vec<String>, Vec<(String, String)>), String)) -> Self {
        let attrs = Attrs::from(&value.0);
        let data = value.1.clone();
        Self { attrs, data }
    }
}

#[derive(Debug)]
pub struct Attrs {
    id: String,
    classes: Vec<String>,
    key_val_pairs: Vec<(String, String)>,
}

impl From<&(String, Vec<String>, Vec<(String, String)>)> for Attrs {
    fn from(attrs: &(String, Vec<String>, Vec<(String, String)>)) -> Self {
        let (id, classes, key_val_pairs) = attrs.clone();
        Self {
            id,
            classes,
            key_val_pairs,
        }
    }
}

#[derive(Diagnostic, Debug, Error)]
pub enum AttrParseError {
    #[error("missing argument `path`")]
    MissingPath,

    #[error("missing argument `age-keys`")]
    MissingAgeKeys,

    #[error("invalid age key")]
    InvalidAgeKey { key_index: usize, error: String },
}

pub struct MarkdownWithAttrs {
    markdown: String,
    attrs: Range<usize>,
    id: Range<usize>,
    classes: BTreeMap<String, Range<usize>>,
    key_val_pairs: BTreeMap<String, Range<usize>>,
}

impl Code {
    pub fn new_if_marked(
        attrs: &(String, Vec<String>, Vec<(String, String)>),
        data: &str,
    ) -> Option<Self> {
        let attrs = Attrs::from(attrs);
        if attrs.is_qrcloak() {
            Some(Self {
                attrs,
                data: data.to_owned(),
            })
        } else {
            None
        }
    }

    pub fn to_markdown(&self) -> MarkdownWithAttrs {
        let mut md = MarkdownWithAttrs {
            attrs: 0..0,
            id: 0..0,
            markdown: String::default(),
            classes: Default::default(),
            key_val_pairs: Default::default(),
        };

        md.markdown.push_str("```");

        self.attrs.to_markdown_into(&mut md);

        md.markdown.push_str(&format!("\n{}\n```\n", self.data));

        md
    }

    pub fn parse(&self) -> Result<CodeOpts> {
        let opts = self.attrs.parse()
                    .map_err(|e| {

                        let code_block = self.to_markdown();

                        let here_span = SourceSpan::new(code_block.attrs.end.into(), 0);

                        let block_span = SourceSpan::new(0.into(), code_block.markdown.len());

                        let src = NamedSource::new("", code_block.markdown.clone()).with_language("Markdown");

                        match e {
                            AttrParseError::MissingPath => {
                                FilterError::MissingArgument {
                                    argument_name: "path".into(),
                                    advice: "add a `path` field to the code block where the QR code will be saved".into(),
                                    block_span,
                                    here_span,
                                    src,
                                }
                            }
                            AttrParseError::MissingAgeKeys => {
                                FilterError::MissingArgument {
                                    argument_name: "age-keys".into(),
                                    advice: "add a `age-keys` field to the code block to specify the recipients of the QR code".into(),
                                    block_span,
                                    here_span,
                                    src,
                                }
                            }

                            AttrParseError::InvalidAgeKey { key_index, error } => {
                                let r = code_block.key_val_pairs.get("age-keys").expect("should exist");

                                let (_, mut vals) = code_block.markdown[r.clone()].split_once('=').expect("should split");

                                vals = &vals[1..vals.len() - 1];

                                let mut pos = r.start + "age-keys=\"".len();

                                for (_, val) in vals.split(',').enumerate().take_while(|(i, _)| *i < key_index) {
                                    pos += val.len() + 1;
                                }
                                let invalid_value = vals.split(',').nth(key_index).expect("should exist");



                                FilterError::InvalidArgument {
                                    argument_name: "age-keys".into(), 
                                    spans: vec![LabeledSpan::new(Some(error), pos, invalid_value.len())],
                                    advice: "make sure the age key is valid".into(),
                                    block_span,
                                    src,
                                }
                            }
                        }
                    })?;

        let data = if let Some(data_cmd) = &opts.data_cmd {
            let mut child = Command::new(data_cmd)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .into_diagnostic()?;

            child
                .stdin
                .as_mut()
                .expect("set to pipe")
                .write_all(self.data.as_bytes())
                .into_diagnostic()?;

            let output = child.wait_with_output().into_diagnostic()?;

            if !output.status.success() {
                let mut err = miette!("data command failed");

                if let Ok(default) = String::from_utf8(output.stderr) {
                    err = err.wrap_err(default);
                }

                return Err(err.wrap_err(self.data.clone()));
            }

            if let Ok(out) = from_utf8(&output.stdout) {
                out.to_string()
            } else {
                return Err(miette!("data command output was not valid utf8"));
            }
        } else {
            self.data.clone()
        };

        Ok(CodeOpts { data, attr: opts })
    }
}

impl Attrs {
    pub fn is_qrcloak(&self) -> bool {
        self.id == "qrcloak"
    }

    fn to_markdown_into(&self, markdown: &mut MarkdownWithAttrs) {
        let text = &mut markdown.markdown;

        let attrs_start = text.len();

        text.push('{');
        let id_start = text.len();
        text.push_str(&format!("#{}", self.id));

        markdown.id = id_start..text.len();

        for class in &self.classes {
            let class_start = text.len() + 1;
            text.push_str(&format!(" .{}", class));
            markdown
                .classes
                .insert(class.to_string(), class_start..text.len());
        }

        for (key, val) in &self.key_val_pairs {
            let key_start = text.len() + 1;
            text.push_str(&format!(" {key}=\"{val}\""));
            markdown
                .key_val_pairs
                .insert(key.to_string(), key_start..text.len());
        }
        text.push('}');

        markdown.attrs = attrs_start..text.len();
    }

    pub fn parse(&self) -> std::result::Result<AttrOpts, AttrParseError> {
        let mut path = None;
        let mut alt_name = None;
        let mut data_cmd = None;
        let mut age_keys = vec![];

        let mut leftover_key_val_pairs = vec![];

        for (key, val) in &self.key_val_pairs {
            match key.as_ref() {
                "data-cmd" => data_cmd = Some(val.to_string()),
                "alt-name" => alt_name = Some(val.to_string()),
                "path" => path = Some(val),
                "age-keys" => {
                    age_keys = val
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .enumerate()
                        .try_fold(Vec::new(), |mut acc, (i, s)| {
                            let recipient = age::x25519::Recipient::from_str(s).map_err(|e| {
                                AttrParseError::InvalidAgeKey {
                                    error: e.into(),
                                    key_index: i,
                                }
                            })?;
                            acc.push(recipient);
                            Ok(acc)
                        })?
                }
                _ => leftover_key_val_pairs.push((key.to_string(), val.to_string())),
            }
        }

        let path = if let Some(path) = path {
            path.to_string()
        } else {
            return Err(AttrParseError::MissingPath);
        };

        if age_keys.is_empty() {
            return Err(AttrParseError::MissingAgeKeys);
        }

        Ok(AttrOpts {
            path,
            age_keys,
            alt_name,
            data_cmd,
            leftover_classes: self.classes.clone(),
            leftover_key_val_pairs,
        })
    }
}

pub struct CodeOpts {
    data: String,
    attr: AttrOpts,
}

impl CodeOpts {
    pub fn eval_to_image(&self) -> Result<pandoc_ast::Inline> {
        let payload = qrcloak_core::payload::PayloadBuilder::default()
            .with_encryption(Some(qrcloak_core::payload::EncryptionOptions::AgeKey(
                AgeKeyOptions::new(&self.attr.age_keys),
            )))
            .build(&self.data)
            .into_diagnostic()?;

        let qrcode = qrcloak_core::generate::Generator::default()
            .generate(&payload)
            .into_diagnostic()?;

        qrcode.first().save(&self.attr.path).into_diagnostic()?;

        Ok(pandoc_ast::Inline::Image(
            (
                "qrcloak".to_string(),
                self.attr.leftover_classes.clone(),
                self.attr.leftover_key_val_pairs.clone(),
            ),
            Default::default(),
            (
                self.attr.path.clone(),
                self.attr.alt_name.clone().unwrap_or_default(),
            ),
        ))
    }
}

pub struct AttrOpts {
    path: String,
    age_keys: Vec<Recipient>,
    alt_name: Option<String>,
    data_cmd: Option<String>,
    leftover_classes: Vec<String>,
    leftover_key_val_pairs: Vec<(String, String)>,
}

mod recursive;

fn filter_pandoc(mut pandoc: Pandoc) -> Result<Pandoc> {
    let codes = recursive::codes(pandoc.blocks.iter_mut());

    for code in codes {
        let opts = code.code.parse()?;

        *code.inline = opts.eval_to_image()?;
    }

    Ok(pandoc)
}

pub fn filter(input: String) -> Result<String> {
    let mut error = None;

    let output = pandoc_ast::filter(input, |pandoc| {
        let old_pandoc = pandoc.clone();

        match filter_pandoc(pandoc) {
            Ok(pandoc) => pandoc,
            Err(e) => {
                error = Some(e);
                old_pandoc
            }
        }
    });

    if let Some(e) = error {
        return Err(e);
    }

    Ok(output)
}

use std::io::{stdin, Read};

use miette::highlighters::SyntectHighlighter;

fn main() -> Result<()> {
    miette::set_panic_hook();
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .color(true)
                .with_syntax_highlighting(SyntectHighlighter::default())
                .build(),
        )
    }))?;

    let mut input = String::new();
    stdin().read_to_string(&mut input).into_diagnostic()?;

    let output = filter(input)?;

    println!("{}", output);

    Ok(())
}
