// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]
extern crate sgx_tstd as std;
extern crate sgx_rand as rand;

#[cfg(target_env = "sgx")]
extern crate core;

#[macro_use] pub mod crypto;
pub mod file;
pub mod metadata;
pub mod types;
pub mod utils;
pub mod test;

pub mod driver {
    use std::vec::Vec;
    use std::path::PathBuf;
    use std::io::Read;
    use std::sgxfs::SgxFile;
    use std::untrusted::fs::File;

    use crate::file::FileHandle;
    use crate::metadata::{EncryptionSchema, encode_secretbox_cipher};
    use crate::types::{
        crypto::{BoxSecretKey, CryptoError},
        ipfs::{CID},
        file::{DEFAULT_CHUNK_SIZE, Hash, FileError},
        driver::{ProtocolError},
        metadata::{
            PreSeal, MetadataError,
        }
    };
    use crate::crypto;
    use crate::utils::{pad_usize};
    use crate::metadata::RecordStore;

    fn file_error(error: FileError) -> ProtocolError {
        ProtocolError::FileError(error)
    }

    fn crypto_error(error: CryptoError) -> ProtocolError {
        ProtocolError::CryptoError(error)
    }

    fn metadata_error(error: MetadataError) -> ProtocolError {
        ProtocolError::MetadataError(error)
    }

    fn read_bytes<R>(reader: R, limit: u64) -> Result<(Vec<u8>, usize), FileError>
    where 
        R:Read
    {
        let mut buf = Vec::new();
        let mut chunk = reader.take(limit);
        let n = chunk.read_to_end(&mut buf).map_err(|_| FileError::FileNotFound)?;
        Ok((buf, n))
    }

    // generate processed & encrypted chunks 
    pub fn pre_upstream(path: &PathBuf, id: &Hash, output_path: &PathBuf, records: &mut RecordStore) -> Result<(), ProtocolError> {
        
        let sealing_key = random_bytes!(32);
        let mut hash: Option<Hash> = None;
        let mut file = SgxFile::open(&path).map_err(|_| ProtocolError::FileError(FileError::FileNotFound))?;
        
        loop {
            // read current chunk
            let (buf, len) = read_bytes(&mut file, DEFAULT_CHUNK_SIZE as u64).map_err(file_error)?;

            if len == 0 {
                break;
            }
            
            hash = match hash {
                None => Some(FileHandle::sha256_checksum(&buf)),
                Some(h) => {
                    Some(FileHandle::sha256_checksum(&[
                        &h[..], &buf[..]
                    ].concat()).into())
                }
            };

            let mut chunk = FileHandle::deflate_chunk(&buf);
            chunk = encode_secretbox_cipher(&crypto::NaClSecretBox::encrypt(&sealing_key, &chunk).map_err(crypto_error)?);
            // chunks.push(chunk);
            
            // record the chunk
            // TODO: handle err for this
            FileHandle::unstrusted_write(
                output_path, 
                &[&pad_usize(chunk.len())[..], &chunk[..]].concat(),
                true //append
            );
        };
        
        records.push(&id, &hash.unwrap(), &sealing_key);
        Ok(())
    }

    // take in IPFS upload response and return the serialized form of sealedMetadata
    pub fn post_upstream(
        cid_list: &CID,
        records: &RecordStore,
        id: &Hash,
        encryption_schema: &EncryptionSchema,
        output_path: &PathBuf,
    ) -> Result<(), ProtocolError> {
        
        let record = match records.get(&id) {
            Some(r) => r,
            None => return Err(ProtocolError::RecordError)
        };

        let pre_seal = PreSeal {
            chunk_cid: * cid_list,
            hash: record.hash,
            sealing_key: record.sealing_key,
            
            // version code as of @skyekiwi/metadata v0.5.2-10
		    version: [0x0, 0x0, 0x1, 0x1],
        };

        let sealed = crate::metadata::seal(&pre_seal, &encryption_schema).map_err(metadata_error)?;
        let encoded_sealed = crate::metadata::encode_sealed_metadata(&sealed).map_err(metadata_error)?;
        
        // TODO: handle err for this
        FileHandle::unstrusted_write(&output_path, &encoded_sealed, false);
        Ok(())
    }

    pub fn pre_downstream(
        encoded_sealed: &[u8],
        keys: &[BoxSecretKey],
        id: &Hash,
        records: &mut RecordStore,
    ) -> Result<CID, ProtocolError> {
        let sealed = crate::metadata::decode_sealed_metadata(encoded_sealed).map_err(metadata_error)?;
        let pre_seal = crate::metadata::unseal(&sealed, keys).map_err(metadata_error)?;
        
        records.push(&id, &pre_seal.hash, &pre_seal.sealing_key);
        Ok(pre_seal.chunk_cid)
    }

    pub fn post_downstream(
        input_path: &PathBuf,
        output_path: &PathBuf,
        id: &Hash,
        records: &RecordStore,
    ) -> Result<(), ProtocolError> {
        let record = match records.get(&id) {
            Some(r) => r,
            None => return Err(ProtocolError::RecordError)
        };

        let mut hash: Option<Hash> = None;

        let mut all_chunks = Vec::new();
        let mut chunks = File::open(input_path).expect("chunk file must exist");
        let len = chunks.read_to_end(&mut all_chunks).unwrap_or(0);

        let mut offset = 0;
        while offset < len && offset + 4 <= len {
            let size_buf = &all_chunks[offset..offset + 4];
            let size = crate::utils::padded_slice_to_usize(size_buf);

            if offset + 4 + size > len {
                // something is wrong. We should never get in here
                break;
            }

            let chunk = &all_chunks[offset + 4..offset + 4 + size];
            offset += 4 + size;

            let mut buf = crypto::NaClSecretBox::decrypt(
                &record.sealing_key, crate::metadata::decode_secretbox_cipher(&chunk),
            ).map_err(crypto_error).unwrap();
            buf = FileHandle::inflate_chunk(&buf);

            hash = match hash {
                None => Some(FileHandle::sha256_checksum(&buf)),
                Some(h) => {
                    Some(FileHandle::sha256_checksum(&[
                        &h[..], &buf[..]
                    ].concat()).into())
                }
            };

            // TODO: handle err for this
            FileHandle::write(&output_path, &buf).map_err(file_error);
        }
        
        if hash.is_some() && hash == Some(record.hash) {
            Ok(())
        } else {
            Err(ProtocolError::FileError(FileError::HashError))
        }
    }
}