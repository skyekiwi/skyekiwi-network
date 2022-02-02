// Copyright 2021 @skyekiwi authors & contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use reqwest::{
  Error as ReqwestError,
  blocking::{multipart::{Form, Part}},
  StatusCode, header::{HeaderMap, HeaderValue}
};
use serde::Deserialize;
use std::str::Utf8Error;

pub mod ipfs_types {
	pub type Cid = Vec<u8>;
	
	#[derive(Debug, Clone)]
	pub struct IpfsResult {
		pub cid: Cid, 
		pub size: u64
	}

	#[derive(Debug)]
	pub enum IpfsError {
		IpfsAddFailed,
		IpfsPinFailed,
		IpfsCatFailed,
    HttpCallError,
	}
}

use ipfs_types::*;

pub struct IpfsClient ();

#[derive(Deserialize, Debug, Clone)]
struct IpfsResponse {
  Name: String,
  Hash: String,
  Size: String,
}

impl std::convert::From<ReqwestError> for ipfs_types::IpfsError {
  fn from(_: ReqwestError) -> Self {
    ipfs_types::IpfsError::HttpCallError
  }
}
impl std::convert::From<std::str::Utf8Error> for ipfs_types::IpfsError {
  fn from(_: Utf8Error) -> Self {
    ipfs_types::IpfsError::HttpCallError
  }
}

impl IpfsClient {

  pub fn add(content: Vec<u8>) -> Result<IpfsResult, IpfsError> {
    const AUTH_HEADER: &str = "Basic bmVhci03Wm9zdjVIQmRINmNTcGFVQXZmcDZMVjk4clpQMlZhYlI2R2ZpQXlQUGI4UjpmM2ZkNDYwNTM3MDYzYTgyM2VjMzdlNGJmZmNhZTQzMWY3MmYzODhkNmU5MWExMzZkMzNhYzRmODU0N2IwMzE5MjMzMGYxNmQ3NGQ0Y2RmZTIzOWNmY2M4ZGFjZTA1ZWVlMDRjNTkyNGNkOGNhM2I4N2EzNWQ2NjExMjM4MGQwOA==";
    
    let mut add_headers = HeaderMap::new();
    add_headers.insert("Authorization", HeaderValue::from_static(AUTH_HEADER));

    let client = reqwest::blocking::Client::builder()
      .default_headers(add_headers)
      .build()?;

    let mut form = Form::new();
    let part = Part::bytes(content).file_name("skyekiwi-protocol-file");
    form = form.part("file", part);
  
    let add = client.post("https://crustwebsites.net/api/v0/add")
      .multipart(form)
      .send()?;
  
    let result: IpfsResponse = add.json().unwrap();
    
    let cid: Cid = result.Hash.as_bytes().into();
    let size = result.Size.parse::<u64>().unwrap();

    let mut map = std::collections::HashMap::new();
    map.insert("cid", std::str::from_utf8(&cid)?);
    let pin = client.post("https://pin.crustcode.com/psa/pins")
      .form(&map)
      .send()?;

    if pin.status() == StatusCode::OK {
      Ok(IpfsResult {cid, size})
    } else {
      Err(IpfsError::IpfsPinFailed)
    }
  }

  pub fn cat(cid: Cid) -> Result<Vec<u8>, IpfsError> {
    const AUTH_HEADER: &str = "Basic bmVhci03Wm9zdjVIQmRINmNTcGFVQXZmcDZMVjk4clpQMlZhYlI2R2ZpQXlQUGI4UjpmM2ZkNDYwNTM3MDYzYTgyM2VjMzdlNGJmZmNhZTQzMWY3MmYzODhkNmU5MWExMzZkMzNhYzRmODU0N2IwMzE5MjMzMGYxNmQ3NGQ0Y2RmZTIzOWNmY2M4ZGFjZTA1ZWVlMDRjNTkyNGNkOGNhM2I4N2EzNWQ2NjExMjM4MGQwOA==";
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_static(AUTH_HEADER));

    let client = reqwest::blocking::Client::builder()
      .default_headers(headers)
      .build()?;

    let download = client.post("https://crustwebsites.net/api/v0/cat")
      .query(&[("arg", std::str::from_utf8(&cid)?)])
      .send()?;
    
    Ok(download.bytes()?.to_vec())
  }
}

#[test]
fn ipfs_works() {
  const CONTENT: &str = "some random string ...";

  let result = IpfsClient::add(CONTENT.as_bytes().to_vec()).unwrap();
  let recovered = IpfsClient::cat(result.cid).unwrap();

  assert_eq!(recovered, CONTENT.as_bytes());
}
