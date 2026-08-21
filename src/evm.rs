use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use tiny_keccak::{Hasher, Keccak};

#[derive(Clone, Debug, Serialize)]
pub struct Instruction {
    pub offset: usize,
    pub opcode: u8,
    pub mnemonic: String,
    pub immediate: Vec<u8>,
}

impl Instruction {
    pub fn size(&self) -> usize {
        1 + self.immediate.len()
    }

    pub fn display(&self) -> String {
        if self.immediate.is_empty() {
            format!("{:06x}: {}", self.offset, self.mnemonic)
        } else {
            format!(
                "{:06x}: {:<12} 0x{}",
                self.offset,
                self.mnemonic,
                hex::encode(&self.immediate)
            )
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BasicBlock {
    pub start: usize,
    pub end: usize,
    pub instructions: Vec<Instruction>,
    pub edges: Vec<CfgEdge>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CfgEdge {
    pub to: usize,
    pub label: Option<String>,
}

pub fn read_bytecode_file(path: &Path) -> Result<Vec<u8>> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    if bytes.is_empty() {
        bail!("bytecode file is empty");
    }

    if looks_like_hex_text(&bytes) {
        let text = String::from_utf8_lossy(&bytes);
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        let raw = compact.strip_prefix("0x").unwrap_or(&compact);
        if raw.is_empty() {
            bail!("bytecode hex file does not contain data");
        }
        if raw.len() % 2 != 0 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("bytecode text is not valid hexadecimal data");
        }
        return hex::decode(raw).context("failed to decode bytecode hex");
    }

    Ok(bytes)
}

fn looks_like_hex_text(bytes: &[u8]) -> bool {
    if !bytes.iter().all(|b| b.is_ascii()) {
        return false;
    }
    let text = String::from_utf8_lossy(bytes);
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let raw = compact.strip_prefix("0x").unwrap_or(&compact);
    !raw.is_empty() && raw.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn disassemble(code: &[u8]) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    let mut pc = 0usize;

    while pc < code.len() {
        let opcode = code[pc];
        let mnemonic = opcode_name(opcode);
        let immediate_len = if (0x60..=0x7f).contains(&opcode) {
            (opcode - 0x5f) as usize
        } else {
            0
        };
        let available = code.len().saturating_sub(pc + 1);
        let take = immediate_len.min(available);
        let immediate = code[pc + 1..pc + 1 + take].to_vec();

        instructions.push(Instruction {
            offset: pc,
            opcode,
            mnemonic,
            immediate,
        });

        pc += 1 + take;
    }

    instructions
}

pub fn opcode_name(opcode: u8) -> String {
    if (0x60..=0x7f).contains(&opcode) {
        return format!("PUSH{}", opcode - 0x5f);
    }
    if (0x80..=0x8f).contains(&opcode) {
        return format!("DUP{}", opcode - 0x7f);
    }
    if (0x90..=0x9f).contains(&opcode) {
        return format!("SWAP{}", opcode - 0x8f);
    }
    if (0xa0..=0xa4).contains(&opcode) {
        return format!("LOG{}", opcode - 0xa0);
    }

    match opcode {
        0x00 => "STOP",
        0x01 => "ADD",
        0x02 => "MUL",
        0x03 => "SUB",
        0x04 => "DIV",
        0x05 => "SDIV",
        0x06 => "MOD",
        0x07 => "SMOD",
        0x08 => "ADDMOD",
        0x09 => "MULMOD",
        0x0a => "EXP",
        0x0b => "SIGNEXTEND",
        0x10 => "LT",
        0x11 => "GT",
        0x12 => "SLT",
        0x13 => "SGT",
        0x14 => "EQ",
        0x15 => "ISZERO",
        0x16 => "AND",
        0x17 => "OR",
        0x18 => "XOR",
        0x19 => "NOT",
        0x1a => "BYTE",
        0x1b => "SHL",
        0x1c => "SHR",
        0x1d => "SAR",
        0x20 => "KECCAK256",
        0x30 => "ADDRESS",
        0x31 => "BALANCE",
        0x32 => "ORIGIN",
        0x33 => "CALLER",
        0x34 => "CALLVALUE",
        0x35 => "CALLDATALOAD",
        0x36 => "CALLDATASIZE",
        0x37 => "CALLDATACOPY",
        0x38 => "CODESIZE",
        0x39 => "CODECOPY",
        0x3a => "GASPRICE",
        0x3b => "EXTCODESIZE",
        0x3c => "EXTCODECOPY",
        0x3d => "RETURNDATASIZE",
        0x3e => "RETURNDATACOPY",
        0x3f => "EXTCODEHASH",
        0x40 => "BLOCKHASH",
        0x41 => "COINBASE",
        0x42 => "TIMESTAMP",
        0x43 => "NUMBER",
        0x44 => "PREVRANDAO",
        0x45 => "GASLIMIT",
        0x46 => "CHAINID",
        0x47 => "SELFBALANCE",
        0x48 => "BASEFEE",
        0x49 => "BLOBHASH",
        0x4a => "BLOBBASEFEE",
        0x50 => "POP",
        0x51 => "MLOAD",
        0x52 => "MSTORE",
        0x53 => "MSTORE8",
        0x54 => "SLOAD",
        0x55 => "SSTORE",
        0x56 => "JUMP",
        0x57 => "JUMPI",
        0x58 => "PC",
        0x59 => "MSIZE",
        0x5a => "GAS",
        0x5b => "JUMPDEST",
        0x5c => "TLOAD",
        0x5d => "TSTORE",
        0x5e => "MCOPY",
        0x5f => "PUSH0",
        0xf0 => "CREATE",
        0xf1 => "CALL",
        0xf2 => "CALLCODE",
        0xf3 => "RETURN",
        0xf4 => "DELEGATECALL",
        0xf5 => "CREATE2",
        0xfa => "STATICCALL",
        0xfd => "REVERT",
        0xfe => "INVALID",
        0xff => "SELFDESTRUCT",
        _ => return format!("UNKNOWN_{opcode:02X}"),
    }
    .to_string()
}

pub fn function_selector(signature: &str) -> [u8; 4] {
    let hash = keccak256(signature.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
}

pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(data);
    hasher.finalize(&mut out);
    out
}

pub fn discover_push4_selectors(instructions: &[Instruction]) -> BTreeSet<String> {
    instructions
        .iter()
        .filter(|ins| ins.opcode == 0x63 && ins.immediate.len() == 4)
        .map(|ins| format!("0x{}", hex::encode(&ins.immediate)))
        .collect()
}

pub fn eip1967_slot(label: &str) -> String {
    let mut hash = keccak256(label.as_bytes());
    decrement_be(&mut hash);
    format!("0x{}", hex::encode(hash))
}

fn decrement_be(bytes: &mut [u8; 32]) {
    for byte in bytes.iter_mut().rev() {
        if *byte > 0 {
            *byte -= 1;
            break;
        }
        *byte = 0xff;
    }
}

pub fn storage_word_to_address(word: &str) -> Option<String> {
    let raw = word.strip_prefix("0x").unwrap_or(word);
    if raw.len() != 64 || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let address = &raw[24..];
    if address.chars().all(|c| c == '0') {
        return None;
    }
    Some(format!("0x{}", address.to_ascii_lowercase()))
}

pub fn detect_eip1167(code: &[u8]) -> Option<String> {
    const PREFIX: &[u8] = &[0x36, 0x3d, 0x3d, 0x37, 0x3d, 0x3d, 0x3d, 0x36, 0x3d, 0x73];
    const SUFFIX: &[u8] = &[
        0x5a, 0xf4, 0x3d, 0x82, 0x80, 0x3e, 0x90, 0x3d, 0x91, 0x60, 0x2b, 0x57, 0xfd, 0x5b,
        0xf3,
    ];
    if code.len() == 45 && code.starts_with(PREFIX) && code[30..].starts_with(SUFFIX) {
        return Some(format!("0x{}", hex::encode(&code[10..30])));
    }
    None
}

fn terminates_block(opcode: u8) -> bool {
    matches!(opcode, 0x00 | 0x56 | 0x57 | 0xf3 | 0xfd | 0xfe | 0xff)
}

pub fn build_cfg(instructions: &[Instruction]) -> BTreeMap<usize, BasicBlock> {
    if instructions.is_empty() {
        return BTreeMap::new();
    }

    let mut starts = BTreeSet::new();
    starts.insert(instructions[0].offset);

    for (idx, ins) in instructions.iter().enumerate() {
        if ins.opcode == 0x5b {
            starts.insert(ins.offset);
        }
        if terminates_block(ins.opcode) {
            if let Some(next) = instructions.get(idx + 1) {
                starts.insert(next.offset);
            }
        }
    }

    let starts_vec: Vec<usize> = starts.iter().copied().collect();
    let by_offset: BTreeMap<usize, Instruction> = instructions
        .iter()
        .cloned()
        .map(|ins| (ins.offset, ins))
        .collect();

    let mut blocks = BTreeMap::new();
    for (idx, start) in starts_vec.iter().enumerate() {
        let end_exclusive = starts_vec
            .get(idx + 1)
            .copied()
            .unwrap_or_else(|| instructions.last().unwrap().offset + instructions.last().unwrap().size());
        let block_ins: Vec<Instruction> = by_offset
            .range(*start..end_exclusive)
            .map(|(_, ins)| ins.clone())
            .collect();
        if block_ins.is_empty() {
            continue;
        }
        let end = block_ins.last().unwrap().offset + block_ins.last().unwrap().size();
        blocks.insert(
            *start,
            BasicBlock {
                start: *start,
                end,
                instructions: block_ins,
                edges: Vec::new(),
            },
        );
    }

    let valid_starts: BTreeSet<usize> = blocks.keys().copied().collect();
    let block_keys: Vec<usize> = blocks.keys().copied().collect();

    for (idx, start) in block_keys.iter().enumerate() {
        let next_start = block_keys.get(idx + 1).copied();
        let block = blocks.get_mut(start).unwrap();
        let last = block.instructions.last().unwrap();
        let previous = if block.instructions.len() >= 2 {
            block.instructions.get(block.instructions.len() - 2)
        } else {
            None
        };

        match last.opcode {
            0x56 => {
                if let Some(dest) = static_jump_destination(previous) {
                    if valid_starts.contains(&dest) {
                        block.edges.push(CfgEdge { to: dest, label: None });
                    }
                }
            }
            0x57 => {
                if let Some(dest) = static_jump_destination(previous) {
                    if valid_starts.contains(&dest) {
                        block.edges.push(CfgEdge {
                            to: dest,
                            label: Some("true".into()),
                        });
                    }
                }
                if let Some(fallthrough) = next_start {
                    block.edges.push(CfgEdge {
                        to: fallthrough,
                        label: Some("false".into()),
                    });
                }
            }
            opcode if terminates_block(opcode) => {}
            _ => {
                if let Some(fallthrough) = next_start {
                    block.edges.push(CfgEdge {
                        to: fallthrough,
                        label: None,
                    });
                }
            }
        }
    }

    blocks
}

fn static_jump_destination(previous: Option<&Instruction>) -> Option<usize> {
    let ins = previous?;
    if !(0x60..=0x7f).contains(&ins.opcode) || ins.immediate.is_empty() || ins.immediate.len() > 8 {
        return None;
    }
    let mut value = 0usize;
    for byte in &ins.immediate {
        value = value.checked_mul(256)?.checked_add(*byte as usize)?;
    }
    Some(value)
}

pub fn cfg_to_dot(blocks: &BTreeMap<usize, BasicBlock>) -> String {
    let mut out = String::from(
        "digraph evm_cfg {\n  rankdir=LR;\n  graph [bgcolor=\"#000000\"];\n  node [shape=box fontname=\"monospace\" fontcolor=\"#ffffff\" color=\"#666666\" bgcolor=\"#000000\"];\n  edge [fontname=\"monospace\" fontcolor=\"#ffffff\" color=\"#888888\"];\n",
    );

    for block in blocks.values() {
        let label = block
            .instructions
            .iter()
            .map(Instruction::display)
            .collect::<Vec<_>>()
            .join("\\l");
        out.push_str(&format!(
            "  bb_{:x} [label=\"{}\\l\"];\n",
            block.start,
            escape_dot(&label)
        ));
    }

    for block in blocks.values() {
        for edge in &block.edges {
            match &edge.label {
                Some(label) => out.push_str(&format!(
                    "  bb_{:x} -> bb_{:x} [label=\"{}\"];\n",
                    block.start,
                    edge.to,
                    escape_dot(label)
                )),
                None => out.push_str(&format!(
                    "  bb_{:x} -> bb_{:x};\n",
                    block.start, edge.to
                )),
            }
        }
    }

    out.push_str("}\n");
    out
}

fn escape_dot(input: &str) -> String {
    input.replace('"', "\\\"")
}

pub fn normalize_storage_slot(input: &str) -> Result<String> {
    if let Some(raw) = input.strip_prefix("0x") {
        if raw.is_empty() || !raw.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("invalid hexadecimal storage slot: {input}");
        }
        let trimmed = raw.trim_start_matches('0');
        return Ok(if trimmed.is_empty() {
            "0x0".into()
        } else {
            format!("0x{}", trimmed.to_ascii_lowercase())
        });
    }

    let value: u128 = input
        .parse()
        .map_err(|_| anyhow!("storage slot must be decimal or 0x-prefixed hexadecimal"))?;
    Ok(format!("0x{value:x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_matches_erc20_transfer() {
        assert_eq!(hex::encode(function_selector("transfer(address,uint256)")), "a9059cbb");
    }

    #[test]
    fn disassembles_push() {
        let ins = disassemble(&[0x60, 0x80, 0x60, 0x40, 0x52]);
        assert_eq!(ins[0].mnemonic, "PUSH1");
        assert_eq!(ins[0].immediate, vec![0x80]);
        assert_eq!(ins[2].mnemonic, "MSTORE");
    }

    #[test]
    fn implementation_slot_matches_eip1967() {
        assert_eq!(
            eip1967_slot("eip1967.proxy.implementation"),
            "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc"
        );
    }

    #[test]
    fn extracts_address_from_storage_word() {
        let word = "0x0000000000000000000000001111111111111111111111111111111111111111";
        assert_eq!(
            storage_word_to_address(word).unwrap(),
            "0x1111111111111111111111111111111111111111"
        );
    }
}
