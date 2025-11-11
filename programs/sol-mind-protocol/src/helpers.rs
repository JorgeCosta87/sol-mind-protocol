use anchor_lang::prelude::*;
use mpl_core::types::PluginAuthorityPair;
use crate::errors::ErrorCode;


pub fn decoded_core_plugins(plugins: Option<Vec<Vec<u8>>>) -> Result<Option<Vec<PluginAuthorityPair>>>{
    match plugins {
        Some(items) => {
            let mut out = Vec::with_capacity(items.len());
            for bytes in items {
                let pair = PluginAuthorityPair::try_from_slice(&bytes)
                    .map_err(|_| error!(ErrorCode::InvalidPlugin))?;
                out.push(pair);
            }
            Ok(Some(out))
        }
        None => Ok(None),
    }
}
