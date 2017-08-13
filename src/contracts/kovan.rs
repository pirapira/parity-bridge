use web3::types::{FilterBuilder, Address, U256, H256, H160, H520, Bytes, Log};
use ethabi::{Contract, Token};
use error::{Error, ResultExt};
use contracts::{EthereumDeposit, KovanDeposit, KovanWithdraw, KovanCollectSignatures};

pub struct KovanBridge<'a>(pub &'a Contract);

impl<'a> KovanBridge<'a> {
	pub fn deposit_payload(&self, deposit: EthereumDeposit) -> Bytes {
		let function = self.0.function("deposit").expect("to find function `deposit`");
		let params = vec![
			Token::Address(deposit.recipient.0), 
			Token::Uint(deposit.value.0), 
			Token::FixedBytes(deposit.hash.0.to_vec())
		];
		let result = function.encode_call(params).expect("the params to be valid");
		Bytes(result)
	}

	pub fn deposits_filter(&self, address: Address) -> FilterBuilder {
		let event = self.0.event("Deposit").expect("to find event `Deposit`");
		FilterBuilder::default()
			.address(vec![address])
			.topics(Some(vec![H256(event.signature())]), None, None, None)
	}

	pub fn deposit_from_log(&self, log: Log) -> Result<KovanDeposit, Error> {
		let event = self.0.event("Deposit").expect("to find event `Deposit`");
		let decoded = event.decode_log(
			log.topics.into_iter().map(|t| t.0).collect(),
			log.data.0
		)?;

		if decoded.len() != 2 {
			return Err("Invalid len of decoded deposit event".into())
		}

		let mut iter = decoded.into_iter().map(|v| v.value);

		let result = KovanDeposit {
			recipient: iter.next().and_then(Token::to_address).map(H160).chain_err(|| "expected address")?,
			value: iter.next().and_then(Token::to_uint).map(U256).chain_err(|| "expected uint")?,
		};

		Ok(result)
	}

	pub fn withdraws_filter(&self, address: Address) -> FilterBuilder {
		let event = self.0.event("Withdraw").expect("to find event `Withdraw`");
		FilterBuilder::default()
			.address(vec![address])
			.topics(Some(vec![H256(event.signature())]), None, None, None)
	}

	pub fn withdraw_from_log(&self, log: Log) -> Result<KovanWithdraw, Error> {
		let event = self.0.event("Withdraw").expect("to find event `Withdraw`");
		let mut decoded = event.decode_log(
			log.topics.into_iter().map(|t| t.0).collect(),
			log.data.0
		)?;

		if decoded.len() != 2 {
			return Err("Invalid len of decoded deposit event".into())
		}

		let mut iter = decoded.into_iter().map(|v| v.value);

		let result = KovanWithdraw {
			recipient: iter.next().and_then(Token::to_address).map(H160).chain_err(|| "expected address")?,
			value: iter.next().and_then(Token::to_uint).map(U256).chain_err(|| "expected uint")?,
			hash: log.transaction_hash.expect("hash to exist"),
		};

		Ok(result)
	}

	pub fn collect_signatures_payload(&self, signature: H520, withdraw: KovanWithdraw) -> Bytes {
		// TODO: do assert with ecrecover here

		let mut r = vec![0u8; 32];
		r.copy_from_slice(&signature[0..32]);
		let mut s = vec![0u8; 32];
		s.copy_from_slice(&signature[32..64]);
		let mut v = [0u8; 32];
		v[31] = signature.0[64];

		let function = self.0.function("collectSignatures").expect("to find function `collectSignatures`");
		let params = vec![
			Token::Uint(v),
			Token::FixedBytes(r),
			Token::FixedBytes(s),
			Token::Bytes(withdraw.bytes().0),
		];
		let result = function.encode_call(params).expect("the params to be valid");
		Bytes(result)
	}

	pub fn collect_signatures_filter(&self, _address: Address) -> FilterBuilder {
		unimplemented!();
	}

	pub fn collect_signatures_from_log(&self, _log: Log) -> KovanCollectSignatures {
		unimplemented!();
	}
}
