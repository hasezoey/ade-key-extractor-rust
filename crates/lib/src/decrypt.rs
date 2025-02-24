use std::{
	arch::x86_64::__cpuid,
	io::Write,
	process::{
		Command,
		Stdio,
	},
};

use anyhow::Context;
use base64::Engine;
use byteorder::{
	BigEndian,
	WriteBytesExt,
};
use once_cell::sync::Lazy;
use regex::Regex;
use sysinfo::{
	CpuRefreshKind,
	RefreshKind,
};

const DEVICE_KEY_PATH: &str = r"HKEY_CURRENT_USER\Software\Adobe\Adept\Device";
const ACTIVATION_KEY_PATH: &str = r"HKEY_CURRENT_USER\Software\Adobe\Adept\Activation";

/// Create a new instance of [Command]
fn new_command(cmd: &str) -> Command {
	return Command::new(cmd);
}

/// Create a new instance of [Command] with "wine"
#[cfg(not(windows))]
fn new_wine_cmd() -> Command {
	return new_command("wine");
}

/// Execute a command inside wine
#[cfg(not(windows))]
fn do_wine_like_cmd(cmd_i: &str) -> Command {
	// pass-through to direct exec
	// return new_command(cmd);

	// use wine first
	let mut cmd = new_wine_cmd();
	cmd.arg(cmd_i);

	return cmd;
}

/// Execute a command, without wine as we already are in a windows environment
#[cfg(windows)]
fn do_wine_like_cmd(cmd_i: &str) -> Command {
	return new_command(cmd_i);
}

/// execute a given wine command and print surrounding information
fn exec_cmd_and_wait(mut cmd: Command) -> anyhow::Result<String> {
	let cmd_out = cmd
		.stderr(Stdio::null())
		.stdin(Stdio::null())
		.stdout(Stdio::piped())
		.spawn()
		.context("Failed to spawn command")?
		.wait_with_output()
		.context("Failed to wait for output of command")?;

	let as_string = String::from_utf8(cmd_out.stdout).context("Failed converting output to utf8 string")?;

	return Ok(as_string);
}

/// Regex for parsing output from "vol"
static PARSE_SERIAL_REGEX: Lazy<Regex> = Lazy::new(|| {
	return Regex::new(r"(?mi)Volume Serial Number is ([^\r\n]+)").unwrap();
});

#[derive(Debug)]
pub struct DriveInfo {
	/// Drive letter where the system is installed (mostly "C:")
	pub win_system_drive:         String,
	/// The Volume Serial Number of the System Drive
	pub win_system_volume_serial: u32,
}

/// Retrieves and parses all information related to drives
pub fn get_drive_info() -> anyhow::Result<DriveInfo> {
	// exec and parse output
	// wine cmd "/k echo %SystemRoot% && exit"
	// expected output:
	// C:\windows
	// required output:
	// C:\\

	// exec and parse output (replace ROOT with last output's root)
	// wine cmd "/c" "vol ROOT"
	// expected output:
	// Volume in drive c has no label.
	// Volume Serial Number is 4300-0000
	// required output:
	// 1124073472L (convert number to decimal from hex)
	let root_dir: String;
	{
		let mut root_dir_cmd = do_wine_like_cmd("cmd");
		root_dir_cmd.args(["/c echo %SystemRoot%"]);

		let root_dir_out = exec_cmd_and_wait(root_dir_cmd).context("cmd /c echo %SystemRoot%")?;
		root_dir = root_dir_out
			.split('\\')
			.next()
			.ok_or_else(|| return crate::Error::other(format!("Failed to split at \"\\\" with \"{}\"", root_dir_out)))?
			.to_owned();
	}
	info!("Got RootDir \"{root_dir}\"");

	let serial: u32;
	{
		let mut serial_cmd = do_wine_like_cmd("cmd");
		serial_cmd.args(["/c", &format!("vol {root_dir}")]);

		let serial_out = exec_cmd_and_wait(serial_cmd).context("cmd /c vol")?;

		let caps = PARSE_SERIAL_REGEX.captures(&serial_out).ok_or_else(|| {
			return crate::Error::no_captures("Volume Serial Number");
		})?;
		let serial_hex = &caps[1].replace('-', "");

		trace!("Volume serial: {serial_hex}");

		serial = u32::from_str_radix(serial_hex.trim(), 16)?;
	}
	info!("Got Volume Serial \"{serial}\"");

	return Ok(DriveInfo {
		win_system_drive:         root_dir,
		win_system_volume_serial: serial,
	});
}

// This is a old way, but keeping this for reference
// /// Get the vendor-id via `lscpu` linux command
// fn get_vendor_id_lscpu() -> anyhow::Result<String> {
// 	/// Regex for parsing output from "lscpu"
// 	static LSCPU_VENDOR_REGEX: Lazy<Regex> = Lazy::new(|| {
// 		return Regex::new(r"(?mi)Vendor ID:\s+([^\r\n]+)").unwrap();
// 	});

// 	// old way with "lscpu" linux command
// 	let vendor_cmd = new_command("lscpu");
// 	let lscpu_out = exec_cmd_and_wait(vendor_cmd, "lscpu").context("lscpu")?;
// 	let caps = LSCPU_VENDOR_REGEX.captures(&lscpu_out).ok_or_else(|| {
// 		return crate::Error::no_captures("lscpu vendor");
// 	})?;
// 	return Ok(caps[1].to_owned());
// }

#[derive(Debug)]
pub struct CpuInfo {
	/// The Vendor of the CPU
	pub cpu_vendor:       String,
	/// Magic numbers from cpuid
	pub cpu_magic_number: Vec<u8>,
}

/// Retrieves and parses all information related to cpu
pub fn get_cpu_info() -> anyhow::Result<CpuInfo> {
	// exec and parse output
	// lscpu | sed -n "s/Vendor ID:[ \t]*//p"
	// expected output:
	// AuthenticAMD
	// required output:
	// AuthenticAMD

	// exec and parse output
	// cpuid -1 --raw | sed -n "s/0x00000001:[ \t]*//p"
	// expected output:
	//    eax=0x00a20f12 ebx=0x02040800 ecx=0xfff83203 edx=0x178bfbff
	// required output:
	// a20f12 (as binary)

	let vendor: String = {
		let s = sysinfo::System::new_with_specifics(RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()));

		s.cpus().first().unwrap().vendor_id().to_owned()
	};
	info!("Got vendor \"{vendor}\"");

	let cpu_magic_number: Vec<u8>;
	{
		// if x86_64, use cpuid instruction, otherwise try to use cpuid package

		// x86_64
		#[cfg(target_arch = "x86_64")]
		{
			let res = unsafe { __cpuid(0x00001) };
			let eax_bytes = res.eax.to_be_bytes();
			trace!("Raw CPU Magic number: {:#?}", eax_bytes);

			// paranoia check
			if eax_bytes.len() != 4 {
				return Err(crate::Error::assertion_failed(format!(
					"Expected cpuid EAX bytes to be length 4, got {}",
					eax_bytes.len()
				))
				.into());
			}

			// skip first byte, because ADE does not use it
			cpu_magic_number = eax_bytes[1..].to_vec();
		}
	}
	info!("Got CPU magic number \"{:#?}\"", cpu_magic_number);

	return Ok(CpuInfo {
		cpu_magic_number,
		cpu_vendor: vendor,
	});
}

/// Regex for parsing output from "cpuid"
static ADEPT_USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
	return Regex::new(r"(?mi)username\s+REG_SZ\s+([^\r\n]+)").unwrap();
});

/// Try to get the username that Adobe used
fn get_win_username_adept() -> anyhow::Result<String> {
	// exec and parse output
	// wine reg query HKEY_CURRENT_USER\\Software\\Adobe\\Adept\\Device /v username
	// expected output:
	// HKEY_CURRENT_USER\Software\Adobe\Adept\Device
	//     username    REG_SZ    userNameHere
	// required output:
	// userNameHere
	let mut adept_username_cmd = do_wine_like_cmd("reg");
	adept_username_cmd.args([
		"query",
		DEVICE_KEY_PATH,
		// "HKEY_CURRENT_USER\\Software\\Adobe\\Adept\\Device",
		"/v",
		"username",
	]);

	let adept_username_out = exec_cmd_and_wait(adept_username_cmd).context("reg query DEVICE_KEY_PATH username")?;
	let caps = ADEPT_USERNAME_REGEX.captures(&adept_username_out).ok_or_else(|| {
		return crate::Error::no_captures("adept username");
	})?;

	let username = caps[1].to_owned();

	info!("Got username from Adept \"{username}\"");
	return Ok(username);
}

/// Try to get the username from a environment variable
fn get_win_username_echo() -> anyhow::Result<String> {
	// exec and parse output
	// wine cmd "/k echo %username% && exit"
	// expected output:
	// userNameHere
	// required output:
	// userNameHere
	let mut echo_username_cmd = do_wine_like_cmd("cmd");
	echo_username_cmd.args(["/c", "echo", "%username%"]);

	let username = {
		let mut tmp = exec_cmd_and_wait(echo_username_cmd).context("cmd /c echo %username%")?;
		let trim_len = tmp.trim_end().len();
		tmp.truncate(trim_len);
		tmp
	};

	info!("Got username from echo \"{username}\"");
	return Ok(username);
}

/// Get the username from Adobe, and fallback to environment variable if not found
pub fn get_win_username() -> anyhow::Result<String> {
	let adept_res = get_win_username_adept();

	let Err(adept_err) = adept_res else {
		return adept_res;
	};
	info!("Adept username failed {}", adept_err);

	return get_win_username_echo();
}

/// Regex for parsing output from "cpuid"
static ADEPT_PARSE_VALUE_REGEX: Lazy<Regex> = Lazy::new(|| {
	return Regex::new(r"(?mis)^\s+value\s+REG_SZ\s+([^\r\n]+)").unwrap();
});

/// Helper function to parse the "user" reg-entry
fn adept_information_parse_user(val: &str) -> Option<String> {
	let caps = ADEPT_PARSE_VALUE_REGEX
		.captures(val)
		.ok_or_else(|| {
			return crate::Error::no_captures("adept-parse \"user\"");
		})
		.ok()?;
	return Some(caps[1].to_owned());
}

/// Regex for parsing output from "cpuid"
static ADEPT_PARSE_USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
	return Regex::new(r"(?mis)\s+method\s+REG_SZ\s+([^\r\n]+)\r?\n\s+value\s+REG_SZ\s+([^\r\n]+)").unwrap();
});

/// Helper function to parse the "username" reg-entry
fn adept_information_parse_username(val: &str) -> Option<(String, String)> {
	let caps = ADEPT_PARSE_USERNAME_REGEX
		.captures(val)
		.ok_or_else(|| {
			return crate::Error::no_captures("adept-parse \"username\"");
		})
		.ok()?;
	return Some((caps[1].to_owned(), caps[2].to_owned()));
}

/// Helper function to parse the "privateLicenseKey" reg-entry
fn adept_information_parse_key(val: &str) -> Option<String> {
	let caps = ADEPT_PARSE_VALUE_REGEX
		.captures(val)
		.ok_or_else(|| {
			return crate::Error::no_captures("adept-parse \"key\"");
		})
		.ok()?;
	return Some(caps[1].to_owned());
}

/// Regex for parsing output from "cpuid"
static ADEPT_SUBENTRY_FILTER_REGEX: Lazy<Regex> = Lazy::new(|| {
	return Regex::new(
		r"(?mis)\\\d+\s+\(Default\)\s+REG_SZ\s+((?:username)|(?:user)|(?:privateLicenseKey))\r?\n(.+?)\r?\n\r?\n",
	)
	.unwrap();
});

#[derive(Debug)]
struct AdeptInformationSubEntry {
	/// The "urn:uuid" of the used account
	user:     String,
	/// The method & AdobeID that is used (method, id)
	username: (String, String),
	/// The raw key
	key:      String,
}

/// Helper function to parse through all the sub-entries for a "user", "username" and "privateLicenseKey"
fn get_adept_information_subentries(path: &str) -> anyhow::Result<AdeptInformationSubEntry> {
	// find id with "user", "username" & "privateLicenseKey"

	// ine reg query HKEY_CURRENT_USER\\Software\\Adobe\\Adept\\Activation\\0000 /s
	// expected output:
	// (many)
	// find output:
	// REG_SZ certificate

	// let mut adept_sub_reg_cmd = new_wine_cmd();
	// adept_sub_reg_cmd.args([
	// 	"reg", "query", path, // "HKEY_CURRENT_USER\\Software\\Adobe\\Adept\\Activation\\0000",
	// 	"/s",
	// ]);
	let mut adept_sub_reg_cmd = do_wine_like_cmd("reg");
	adept_sub_reg_cmd.args(["query", path, "/s"]);

	let adept_sub_reg_out = exec_cmd_and_wait(adept_sub_reg_cmd).context("reg query adept subentries")?;

	let mut user: Option<String> = None;
	let mut username: Option<(String, String)> = None;
	let mut key: Option<String> = None;

	for cap in ADEPT_SUBENTRY_FILTER_REGEX.captures_iter(&adept_sub_reg_out) {
		let val_type = &cap[1];
		let val = &cap[2];

		trace!("DEBUG type {val_type}");

		match val_type {
			"user" => user = adept_information_parse_user(val),
			"username" => username = adept_information_parse_username(val),
			"privateLicenseKey" => key = adept_information_parse_key(val),
			_ => (),
		}
	}

	let Some(user) = user else {
		return Err(crate::Error::no_adept_reg_key("user").into());
	};

	let Some(username) = username else {
		return Err(crate::Error::no_adept_reg_key("username").into());
	};

	let Some(key) = key else {
		return Err(crate::Error::no_adept_reg_key("privateLicenseKey").into());
	};

	return Ok(AdeptInformationSubEntry { key, user, username });
}

/// Regex for parsing output from "cpuid"
static ADEPT_DEVICE_KEY_REGEX: Lazy<Regex> = Lazy::new(|| {
	return Regex::new(r"(?mis)key\s+REG_BINARY\s+([^\r\n]+)").unwrap();
});
/// Regex for parsing output from "cpuid"
static ADEPT_ACTIVATION_SUBENTRY_REGEX: Lazy<Regex> = Lazy::new(|| {
	return Regex::new(r"(?mi)\\(\d+)\s+\(Default\)\s+REG_SZ\s+credentials").unwrap();
});

#[derive(Debug)]
pub struct AdeptInformation {
	/// The "urn:uuid" of the used account
	pub user:       String,
	/// The method & AdobeID that is used (method, id)
	pub username:   (String, String),
	/// The raw key
	pub key:        String,
	/// The key of the device
	pub device_key: String,
}

/// Search Adept for information
pub fn get_adept_information() -> anyhow::Result<AdeptInformation> {
	// exec and parse output
	// wine reg query HKEY_CURRENT_USER\\Software\\Adobe\\Adept\\Device /v key
	// expected output:
	// HKEY_CURRENT_USER\Software\Adobe\Adept\Device
	// key    REG_BINARY    really_long_hex_string
	// required output:
	// really_long_hex_string (as binary)
	let device_key: String;
	{
		let mut adept_device_key_cmd = do_wine_like_cmd("reg");
		adept_device_key_cmd.args(["query", DEVICE_KEY_PATH, "/v", "key"]);

		let adept_device_key_out = exec_cmd_and_wait(adept_device_key_cmd).context("reg query DEVICE_KEY_PATH key")?;
		let caps = ADEPT_DEVICE_KEY_REGEX.captures(&adept_device_key_out).ok_or_else(|| {
			return crate::Error::no_captures("adept device key");
		})?;
		device_key = caps[1].to_owned();
	}

	// find id with "credentials"
	// exec and parse output
	// ine reg query HKEY_CURRENT_USER\\Software\\Adobe\\Adept\\Activation /s
	// expected output:
	// (many)
	// find output:
	// REG_SZ certificate

	let mut adept_sub_reg_cmd = do_wine_like_cmd("reg");
	adept_sub_reg_cmd.args(["query", ACTIVATION_KEY_PATH, "/s"]);

	let adept_sub_reg_out = exec_cmd_and_wait(adept_sub_reg_cmd)?;
	let caps = ADEPT_ACTIVATION_SUBENTRY_REGEX
		.captures(&adept_sub_reg_out)
		.ok_or_else(|| {
			return crate::Error::no_captures("adept sub-entry list");
		})?;

	let credentails_path = format!("{ACTIVATION_KEY_PATH}\\{}", &caps[1]);

	let sub_info = get_adept_information_subentries(&credentails_path)?;

	return Ok(AdeptInformation {
		device_key,
		key: sub_info.key,
		user: sub_info.user,
		username: sub_info.username,
	});
}

/// Setup the entropy bytes
fn setup_entropy(drive_info: &DriveInfo, cpu_info: &CpuInfo, user: &str) -> anyhow::Result<Vec<u8>> {
	let mut entropy = vec![];
	entropy.write_u32::<BigEndian>(drive_info.win_system_volume_serial)?;
	entropy.write_all(cpu_info.cpu_vendor.as_bytes())?;
	entropy.write_all(&cpu_info.cpu_magic_number)?;
	let user_asbytes = user.as_bytes();
	entropy.write_all(user_asbytes)?;

	if user_asbytes.len() < 13 {
		let pad = 13 - user_asbytes.len();
		let v = vec![0; pad];
		trace!("padding line by {} bytes", pad);

		entropy.write_all(&v)?;
	}

	trace!("Entropy: {:#?}", encode_hex(&entropy));

	return Ok(entropy);
}

/// Probe if the "ade-extract-winapi-bin.exe" binary exists in the current cwd
fn probe_winapi_binary() -> anyhow::Result<()> {
	let bin_path = std::path::Path::new("./ade-extract-winapi-bin.exe");

	if !bin_path.exists() {
		return Err(crate::Error::NoBinary("ade-extract-winapi-bin.exe", std::env::current_dir()?).into());
	}

	return Ok(());
}

/// Decode the given "input" from hex into a [Vec<u8>] Array
fn decode_hex(input: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
	return (0..input.len())
		.step_by(2)
		.map(|i| return u8::from_str_radix(&input[i..i + 2], 16))
		.collect();
}

/// Encode the given "bytes" to a hex [String]
fn encode_hex(bytes: &[u8]) -> String {
	use std::fmt::Write;

	let mut s = String::with_capacity(bytes.len() * 2);
	for &b in bytes {
		s.write_str(&format!("{:02x}", b)).expect("Expected to written");
	}
	return s;
}

/// Regex for parsing output from "cpuid"
static WINAPI_DECRYPTED_REGEX: Lazy<Regex> = Lazy::new(|| {
	return Regex::new(r#"(?m)^decrypted "([^"]+)"$"#).unwrap();
});

/// Decrypt the key with the given information
pub fn decrypt(
	drive_info: &DriveInfo,
	cpu_info: &CpuInfo,
	user: &str,
	adept_info: &AdeptInformation,
	print_info: bool,
) -> anyhow::Result<Vec<u8>> {
	// decrypt "privateLicenseKey" with "keykey"

	trace!(
		"DEBUG {:#?}, {:#?}, {:#?}, {:#?}",
		drive_info,
		cpu_info,
		user,
		adept_info
	);

	let entropy_hex = encode_hex(&setup_entropy(drive_info, cpu_info, user)?);
	let device_key_hex = adept_info.device_key.clone(); // the devicekey is already a hex

	// Print info, so that the "winapi-bin" can be run separately
	if print_info {
		println!("Entropy (hex): \"{}\"", &entropy_hex);
		println!("Device-Key (hex): \"{}\"", device_key_hex);
		println!("Adept-Key (base64): \"{}\"", &adept_info.key);
	}

	trace!("Trying to run winapi-binary");

	probe_winapi_binary()?;

	let mut winapi_cmd = do_wine_like_cmd("ade-extract-winapi-bin.exe");
	winapi_cmd.args([entropy_hex, device_key_hex]);

	let winapi_out = exec_cmd_and_wait(winapi_cmd).context("ade-extract-winapi-bin.exe")?;
	let caps = WINAPI_DECRYPTED_REGEX.captures(&winapi_out).ok_or_else(|| {
		return crate::Error::no_captures("winapi \"decrypted\" output");
	})?;
	let decrypted_key_hex = &caps[1].to_owned();

	let final_key = aes_decrypt(decrypted_key_hex, &adept_info.key)?;

	return Ok(final_key);
}

/// AES decrypt the given "adept_key" with "key_hex"
pub fn aes_decrypt(key_hex: &str, adept_key: &str) -> anyhow::Result<Vec<u8>> {
	let decrypted_key = decode_hex(key_hex).context("Failed to decode key_hex")?;

	if decrypted_key.len() != 16 {
		return Err(crate::Error::assertion_failed(format!(
			"decrypted key is not the proper size, expected 16, got {}",
			decrypted_key.len()
		))
		.into());
	}

	trace!("Trying to decrypt AES-CBC key");

	let adept_key_bytes = base64::engine::general_purpose::STANDARD
		.decode(adept_key)
		.context("Failed to decode base64 adept privateLicenseKey")?;

	use libaes::Cipher;

	let decrypted_key_slice: &[u8; 16] = &decrypted_key[0..16]
		.try_into()
		.context("Failed to convert Vec<u8> to sized slice")?;

	let cipher = Cipher::new_128(decrypted_key_slice);

	let iv = vec![0; decrypted_key_slice.len()];

	let mut final_key = cipher.cbc_decrypt(&iv, &adept_key_bytes);

	// remove the first 25 bytes, because it seems like those are not wanted
	final_key.drain(0..=25);

	return Ok(final_key);
}
