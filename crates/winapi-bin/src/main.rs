/// Decode the given "input" from hex into a [Vec<u8>] Array
#[cfg(windows)]
fn decode_hex(input: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
	return (0..input.len())
		.step_by(2)
		.map(|i| return u8::from_str_radix(&input[i..i + 2], 16))
		.collect();
}

/// Encode the given "bytes" to a hex [String]
#[cfg(windows)]
fn encode_hex(bytes: &[u8]) -> String {
	use std::fmt::Write;

	let mut s = String::with_capacity(bytes.len() * 2);
	for &b in bytes {
		s.write_str(&format!("{:02x}", b)).expect("Expected to written");
	}
	return s;
}

// Exit Codes:
// 0 - Everything is fine
// -1 - Expected exactly 2 arguments
// -2 - Decryption failed
// -100 - Not compiled for windows

#[cfg(not(windows))]
fn main() {
	println!("Binary only works on windows target");
	std::process::exit(-100);
}

#[cfg(windows)]
fn main() {
	let args: Vec<String> = std::env::args().collect();

	if args.len() < 3 || args.len() > 3 {
		eprintln!("Expected 2 arguments");
		std::process::exit(-1);
	}

	let entropy = &args[1];
	let device_key = &args[2];

	eprintln!("entropy: {:#?}", entropy);
	eprintln!("device: {:#?}", device_key);

	let mut entropy = decode_hex(entropy).expect("Expected to successfully decode entropy from hex");
	let mut device_key_string = decode_hex(device_key).expect("Expected to successfully decode data from hex");

	let out_string: String;

	unsafe {
		let mut blob_in = winapi::um::wincrypt::CRYPTOAPI_BLOB {
			cbData: (device_key_string.len() + 1) as u32,
			pbData: device_key_string.as_mut_ptr(),
		};
		let mut blob_out = winapi::um::wincrypt::CRYPTOAPI_BLOB::default();
		let mut entropy_blob = winapi::um::wincrypt::CRYPTOAPI_BLOB {
			cbData: entropy.len() as u32,
			pbData: entropy.as_mut_ptr(),
		};

		eprintln!("before CryptUnprotectData");

		let res = winapi::um::dpapi::CryptUnprotectData(
			&mut blob_in,
			std::ptr::null_mut(),
			&mut entropy_blob,
			std::ptr::null_mut(),
			std::ptr::null_mut(),
			0,
			&mut blob_out,
		);

		if res == 0 {
			eprintln!("Decryption failed");
			std::process::exit(-2);
		}

		eprintln!("res {res}");

		eprintln!("out1 {:#?}", blob_out.cbData);
		eprintln!("out2 {:#?}", blob_out.pbData);

		let out_bytes = std::slice::from_raw_parts(blob_out.pbData, blob_out.cbData as usize);

		out_string = encode_hex(out_bytes);
	}

	println!("decrypted {:#?}", out_string);

	std::process::exit(0);
}
