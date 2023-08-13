use gumdrop::Options;

#[derive(Debug, Options)]
pub struct DecoderOptions {
    #[options(help = "Print this message")]
    help: bool,

    #[options(help = "The alphabet for the this cipher. For example: ABCDEFGHIKLMNOPQRSTUVWXYZ")]
    pub alphabet: Option<String>,
    #[options(
        help = "The password for this cipher. If omitted, the alphabet will be used unchanged as the password."
    )]
    pub password: Option<String>,

    #[options(help = "The file to decode.")]
    pub input: Option<String>,
    #[options(help = "Where to write the decoded output.")]
    pub output: Option<String>,
    #[options(help = "If specified, extract this file from the abyss.")]
    pub extract_name: Option<String>,

    #[options(help = "What shape is this cipher? Example: 5x5 is bifid")]
    pub shape: Option<String>,
    #[options(help = "")]
    pub group_size: Option<u64>,
}
