/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use ironfish_rust::IncomingViewKey;
use ironfish_rust::MerkleNoteHash;
use ironfish_rust::OutgoingViewKey;
use napi::bindgen_prelude::*;
use napi::JsBuffer;
use napi_derive::napi;

use ironfish_rust::MerkleNote;

#[napi(js_name = "NoteEncrypted")]
pub struct NativeNoteEncrypted {
    pub(crate) note: MerkleNote,
}

#[napi]
impl NativeNoteEncrypted {
    #[napi(constructor)]
    pub fn new(js_bytes: JsBuffer) -> Result<Self> {
        let bytes = js_bytes.into_value()?;
        let note =
            MerkleNote::read(bytes.as_ref()).map_err(|err| Error::from_reason(err.to_string()))?;

        Ok(NativeNoteEncrypted { note })
    }

    #[napi]
    pub fn serialize(&self) -> Result<Buffer> {
        let mut vec: Vec<u8> = vec![];
        self.note
            .write(&mut vec)
            .map_err(|err| Error::from_reason(err.to_string()))?;

        Ok(Buffer::from(vec))
    }

    #[napi]
    pub fn equals(&self, other: &NativeNoteEncrypted) -> bool {
        self.note.eq(&other.note)
    }

    #[napi]
    pub fn merkle_hash(&self) -> Result<Buffer> {
        let mut vec: Vec<u8> = Vec::with_capacity(32);
        self.note
            .merkle_hash()
            .write(&mut vec)
            .map_err(|err| Error::from_reason(err.to_string()))?;

        Ok(Buffer::from(vec))
    }

    /// Hash two child hashes together to calculate the hash of the
    /// new parent
    #[napi]
    pub fn combine_hash(depth: i64, js_left: JsBuffer, js_right: JsBuffer) -> Result<Buffer> {
        let left = js_left.into_value()?;
        let right = js_right.into_value()?;

        let left_hash = MerkleNoteHash::read(left.as_ref())
            .map_err(|err| Error::from_reason(err.to_string()))?;

        let right_hash = MerkleNoteHash::read(right.as_ref())
            .map_err(|err| Error::from_reason(err.to_string()))?;

        let converted_depth: usize = depth
            .try_into()
            .map_err(|_| Error::from_reason("Value could not fit in usize".to_string()))?;

        let mut vec = Vec::with_capacity(32);

        MerkleNoteHash::new(MerkleNoteHash::combine_hash(
            converted_depth,
            &left_hash.0,
            &right_hash.0,
        ))
        .write(&mut vec)
        .map_err(|err| Error::from_reason(err.to_string()))?;

        Ok(Buffer::from(vec))
    }

    /// Returns undefined if the note was unable to be decrypted with the given key.
    #[napi]
    pub fn decrypt_note_for_owner(&self, incoming_hex_key: String) -> Result<Option<Buffer>> {
        let incoming_view_key = IncomingViewKey::from_hex(&incoming_hex_key)
            .map_err(|err| Error::from_reason(err.to_string()))?;

        Ok(match self.note.decrypt_note_for_owner(&incoming_view_key) {
            Ok(note) => {
                let mut vec = vec![];
                note.write(&mut vec)
                    .map_err(|err| Error::from_reason(err.to_string()))?;
                Some(Buffer::from(vec))
            }
            Err(_) => None,
        })
    }

    /// Returns undefined if the note was unable to be decrypted with the given key.
    #[napi]
    pub fn decrypt_note_for_spender(&self, outgoing_hex_key: String) -> Result<Option<Buffer>> {
        let outgoing_view_key = OutgoingViewKey::from_hex(&outgoing_hex_key)
            .map_err(|err| Error::from_reason(err.to_string()))?;
        Ok(
            match self.note.decrypt_note_for_spender(&outgoing_view_key) {
                Ok(note) => {
                    let mut vec = vec![];
                    note.write(&mut vec)
                        .map_err(|err| Error::from_reason(err.to_string()))?;
                    Some(Buffer::from(vec))
                }
                Err(_) => None,
            },
        )
    }
}
