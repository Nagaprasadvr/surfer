use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum MintInstructions {
    InitializeMint,
    SetAuthority,
    MintTo,
    MintToChecked,
    InitilaizeMint2,
}

impl MintInstructions {
    pub fn to_select_vec() -> Vec<&'static str> {
        vec![
            "InitializeMint",
            "SetAuthority",
            "MintTo",
            "MintToChecked",
            "InitilaizeMint2",
        ]
    }

    pub fn from_select_str(select_str: &str) -> anyhow::Result<Self> {
        match select_str {
            "InitializeMint" => Ok(Self::InitializeMint),
            "SetAuthority" => Ok(Self::SetAuthority),
            "MintTo" => Ok(Self::MintTo),
            "MintToChecked" => Ok(Self::MintToChecked),
            "InitilaizeMint2" => Ok(Self::InitilaizeMint2),

            _ => Err(anyhow::anyhow!("Invalid mint instruction: {}", select_str)),
        }
    }
}
