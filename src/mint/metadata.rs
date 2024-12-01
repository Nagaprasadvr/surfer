use mpl_token_metadata::accounts::{MasterEdition, Metadata};
use solana_account::ReadableAccount;
use solana_client::nonblocking::rpc_client::RpcClient;
use spl_pod::solana_pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub metadata: Option<Metadata>,
    pub master_edition: Option<MasterEdition>,
}

impl TokenMetadata {
    pub async fn fetch_and_parse(mint_pubkey: Pubkey, rpc: &RpcClient) -> Option<TokenMetadata> {
        let (mut metadata, master_edition) = tokio::join!(
            fetch_and_parse_metadata(mint_pubkey, rpc),
            fetch_and_parse_master_edition(mint_pubkey, rpc)
        );

        metadata = metadata.map(|mut m| {
            m.uri = m.uri.trim_end_matches('\0').to_string();
            m.name = m.name.trim_end_matches('\0').to_string();
            m.symbol = m.symbol.trim_end_matches('\0').to_string();
            m
        });

        Some(Self {
            metadata,
            master_edition,
        })
    }
}

pub async fn fetch_and_parse_metadata(mint_pubkey: Pubkey, rpc: &RpcClient) -> Option<Metadata> {
    let metadata_pubkey = Metadata::find_pda(&mint_pubkey).0;
    rpc.get_account(&metadata_pubkey)
        .await
        .and_then(|acc| Metadata::from_bytes(acc.data()).map_err(|e| e.into()))
        .ok()
}

pub async fn fetch_and_parse_master_edition(
    mint_pubkey: Pubkey,
    rpc: &RpcClient,
) -> Option<MasterEdition> {
    let master_edition_pubkey = MasterEdition::find_pda(&mint_pubkey).0;

    rpc.get_account(&master_edition_pubkey)
        .await
        .and_then(|acc| MasterEdition::from_bytes(acc.data()).map_err(|e| e.into()))
        .ok()
}
