#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
use anyhow::{Context,};
use core::slice;
use encoder_lib::Key;
use rust_baltech_sdk_bindings::{Buf, *};
use std::collections::HashMap;

pub fn is_selected() -> bool {
    let select_parm = vhl_select_t {
        cardfamiliesfilter: Buf::new(card_families {
            Iso14443A: true,
            ..Default::default()
        }),
        ..Default::default()
    };

    vhl_select(select_parm).is_ok()
}

pub fn try_auth_desfire() -> anyhow::Result<HashMap<&'static str, Box<[u8]>>> {
    let mut key = Key::from(0_u64);

    let p = desfire_authextkey_t {
        securemessaging: Buf::new(securemessaging_native),
        deskeynr: Buf::new(0),
        cryptomode: Buf::new(cryptomode_des),
        key: Buf::new(key.as_mut_ptr()),
        key_len: Buf::new(key.len() as size_t),
        ..Default::default()
    };

    let p2: desfire_authextkey_t = Default::default();

    let get_snr_parm = vhl_getsnr_t {
        ..Default::default()
    };

    let get_atr_parm = vhl_getatr_t {
        ..Default::default()
    };

    let select_app_parm = desfire_selectapplication_t {
        appid: Buf::new(0 as c_uint),
    };

    let snr_resp = vhl_getsnr(get_snr_parm).with_context(|| format!("Failed get SNR"))?;

    let atr_resp = vhl_getatr(get_atr_parm).with_context(|| format!("Failed get ATR"))?;

    desfire_selectapplication(select_app_parm).with_context(|| format!("Failed Select App"))?;

    desfire_authextkey(p).with_context(|| format!("Failed AuthExtKey"))?;

    // SAFETY: atr_len is initialized to zero so no accidental out of bounds reads
    let atr = unsafe {
        slice::from_raw_parts(
            atr_resp.atr.0.assume_init(),
            atr_resp.atr_len.0.assume_init() as usize,
        )
    };
    let snr = unsafe {
        slice::from_raw_parts(
            snr_resp.snr.0.assume_init(),
            snr_resp.snr_len.0.assume_init() as usize,
        )
    };
    let mut result = HashMap::new();
    result.insert("serial_number", snr.to_vec().into_boxed_slice());
    result.insert("atr", atr.to_vec().into_boxed_slice());
    Ok(result)
}
