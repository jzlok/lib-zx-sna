// lib-zx-sna is Copyright (c) 2025 Jez Sherlock

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
// https://opensource.org/license/mit

use std::io::Read;
use std::fs::File;

/// This module provides functionality to handle ZX Spectrum snapshots.
/// It includes structures to represent the snapshot header, extension,
/// and the snapshot itself. The snapshots can be created from binary data
/// or from a file. The module also provides methods to peek and poke
/// memory addresses within the snapshot, allowing for reading and writing
/// of memory values as needed.

const MEM_1K: usize = 1024;
const MEM_16K: usize = MEM_1K * 16;
const MEM_48K: usize = MEM_1K * 48;

#[derive(PartialEq,Debug)]
pub enum SnapshotType {
    Snapshot48,
    Snapshot128,
}

/// Represents the header of a ZX Spectrum snapshot.
/// This struct contains the CPU registers and other state information.
/// It includes the I register, the prime registers (HL', DE', BC', AF'),
/// the main registers (HL, DE, BC, AF), the interrupt mode, and the
/// border color.
/// The header is used to store the state of the ZX Spectrum at the time
/// the snapshot was taken.
/// The fields are represented in little-endian format, which is the
/// standard for ZX Spectrum snapshots.
#[repr(C,packed)]
pub struct SnapshotHeader{
    pub i: u8,
    pub hl_prime: u16,
    pub de_prime: u16,
    pub bc_prime: u16,
    pub af_prime: u16,
    pub hl: u16,
    pub de: u16,
    pub bc: u16,
    pub iy: u16,
    pub ix: u16,
    pub interrupt: u8,
    pub r: u8,
    pub af: u16,
    pub sp: u16,
    pub int_mode: u8,
    pub border_color: u8,
}

impl Default for SnapshotHeader {
    fn default() -> Self {
        SnapshotHeader {
            i: 0,
            hl_prime: 0,
            de_prime: 0,
            bc_prime: 0,
            af_prime: 0,
            hl: 0,
            de: 0,
            bc: 0,
            iy: 0,
            ix: 0,
            interrupt: 0,
            r: 0,
            af: 0,
            sp: 0,
            int_mode: 0,
            border_color: 0,
        }
    }
}

/// Represents an optional extension for the snapshot.
/// This struct contains additional fields for the ZX Spectrum 128 snapshot.
/// It includes the program counter, the 7FFD register, and the TR-DOS state
#[repr(C,packed)]
pub struct SnapshotExtension {
    pub pc: u16,
    pub x7ffd: u8,
    pub tr_dos: u8,
}

/// Represents a snapshot of a ZX Spectrum state.
/// This struct contains the snapshot type, header, optional extension,
/// and a pointer to the memory block representing the snapshot.
#[repr(C)]
pub struct Snapshot{
    pub snapshot_type: SnapshotType,            // type of snapshot (48K or 128K)
    pub header: SnapshotHeader,                 // snapshot header containing CPU state
    pub extension: Option<SnapshotExtension>,   // optional extension for ZX Spectrum 128 snapshots
    pub banks: Vec<Vec<u8>>,                    // banks of memory
    pub mapping: [u8; 3],
}

impl Default for Snapshot {
    fn default() -> Self {
        Snapshot {
            snapshot_type: SnapshotType::Snapshot48,
            header: SnapshotHeader::default(),
            extension: None,
            banks: Vec::new(),
            mapping: [0u8; 3],
        }
    }
}

impl Snapshot {
    /// poke writes a byte to the memory MAPPED to the given address.
    /// If the address is less than 0x4000, it panics with an error message.
    /// The address is expected to be in the range of 0x4000 to 0xFFFF.
    pub fn poke(&mut self, address: u16, value: u8) {
        if address < 0x4000 {
            panic!("Attempted to poke at address < 0x4000, which is invalid.");
        }

        let bank_index = ((address >> 14) & 0x03 ) - 1;
        self.banks[self.mapping[bank_index as usize] as usize][(address & 0x3FFF) as usize] = value;
    }

    /// peek reads a byte from the memory MAPPED to the given address.
    /// If the address is less than 0x4000, it returns 0x
    pub fn peek(&self, address: u16) -> u8 {
        if address < 0x4000 {
            return 0xFF;
        }

        let bank_index = ((address >> 14) & 0x03 ) - 1;
        self.banks[self.mapping[bank_index as usize] as usize][(address & 0x3FFF) as usize]
    }

    /// peek reads a byte from the memory MAPPED to the given address.
    /// If the address is less than 0x4000, it returns 0x
	pub fn peek_word(&self, address:u16) -> u16 {
		if address == 0xFFFF {
			panic!("Attempted to peek16 at address 0xFFFF, which is invalid.");
		}
		(self.peek(address) as u16) | ((self.peek(address+1) as u16) << 8)
	}

    /// poke_word writes a 16-bit value to the memory MAPPED to the given address.
    /// If the address is 0xFFFF, it panics with an error message.
    /// This is a little-endian write operation.
    pub fn poke_word(&mut self, address:u16, value:u16) {
        if address == 0xFFFF {
            panic!("Attempted to poke16 at address 0xFFFF, which is invalid.");
        }
        self.poke(address, (value & 0xFF) as u8);
        self.poke(address + 1, ((value >> 8) & 0xFF) as u8);
    }

    /// changes the bank that is mapped into 0xC000-0xCFFF when using peek (or the future poke) functions.
    pub fn write_0x7ffd(&mut self, value: u8) {
        if self.snapshot_type != SnapshotType::Snapshot128 {
            panic!("Attempted to write to 0x7ffd on a 48K snapshot, which is invalid.");
        }
        self.extension.as_mut().expect("Extension is None").x7ffd = value;
        self.mapping[2] = value & 0x07; // update the mapping based on the new value
    }

    /// bank_peek reads a byte from the specified bank at the given address.
    /// The bank index should be within the range of available banks.
    /// The address is masked to ensure it is within the valid range for the bank.
    /// If the bank index is out of bounds, it panics with an error message.
    pub fn bank_peek(&self, bank: usize, address: u16) -> u8 {
        if bank >= self.banks.len() {
            panic!("Bank index out of bounds");
        }
        self.banks[bank][(address & 0x3FFF) as usize]
    }

    /// bank_poke writes a byte to the specified bank at the given address.
    /// The bank index should be within the range of available banks.
    /// The address is masked to ensure it is within the valid range for the bank.
    /// If the bank index is out of bounds, it panics with an error message.
    /// This function is used to modify the contents of a specific bank in the snapshot.
    pub fn bank_poke(&mut self, bank: usize, address: u16, value: u8) {
        if bank >= self.banks.len() {
            panic!("Bank index out of bounds");
        }
        self.banks[bank][(address & 0x3FFF) as usize] = value;
    }

    /// bank_poke_word writes a 16-bit value to the specified bank at the given address.
    /// The bank index should be within the range of available banks.
    /// Given this is a banked poke, for pokes at 0x3FFF it will poke into the next bank
    /// or wrap around to address 0 of the same bank depending on the wrap parameter.
    pub fn bank_poke_word(&mut self, mut bank: usize, mut address: u16, value: u16, wrap: bool) {
        if bank >= self.banks.len() {
            panic!("Bank index out of bounds");
        }
        self.bank_poke(bank, address, (value & 0xFF) as u8);
        if address == 0xFFFF {
            address = 0;
            if !wrap {
                bank += 1;
            }
        } else {
            address += 1;
        }

        self.bank_poke(bank, address, ((value >> 8) & 0xFF) as u8);
    }

    /// bank_peek_word reads a 16-bit value from the specified bank at the given address.
    /// The bank index should be within the range of available banks.
    /// Given this is a banked peek, for peeks at 0x3FFF it will peek into the next bank
    /// or wrap around to address 0 of the same bank depending on the wrap parameter.
    pub fn bank_peek_word(&mut self, mut bank: usize, mut address: u16, wrap: bool) -> u16 {
        if bank >= self.banks.len() {
            panic!("Bank index out of bounds");
        }
        let low = self.bank_peek(bank, address);
        if address == 0xFFFF {
            address = 0;
            if !wrap {
                bank += 1;
            }
        } else {
            address += 1;
        }

        let high = self.bank_peek(bank, address);
        (high as u16) << 8 as u16 | low as u16
    }

    /// checksum calculates the checksum for a specific bank.
    /// It sums up all the bytes in the specified bank and returns the result as a u16.
    /// The checksum is calculated by iterating through each byte in the bank,
    /// adding it to a running total, and wrapping the sum to prevent overflow.
    /// If the bank index is out of bounds, it panics with an error message.
    #[allow(dead_code)]
    fn checksum(&self, bank:usize) -> u16 {
        if bank >= self.banks.len() {
            panic!("Bank index out of bounds");
        }
        let mut sum: u16 = 0;
        for byte in &self.banks[bank] {
            sum = sum.wrapping_add(*byte as u16);
        }
        sum
    }
}



impl TryFrom<File> for Snapshot {
    type Error = std::io::Error;

    /// Creates a new `Snapshot` from a file.
    /// It reads the binary data from the file and initializes the snapshot.
    /// The file should contain a ZX Spectrum snapshot in binary format.
    fn try_from(mut file: File) -> Result<Self, Self::Error> {
        let mut bin = Vec::new();
        file.read_to_end(&mut bin)?;
        let snapshot = Snapshot::try_from(bin).expect("Failed to create snapshot from binary data");
        Ok(snapshot)
    }
}


impl TryFrom<Vec<u8>> for Snapshot {
    type Error = std::string::FromUtf8Error;

    /// Creates a new `Snapshot` from a binary slice.
    /// It initializes the snapshot based on the binary data provided.
    /// The binary data should be in the format of a ZX Spectrum snapshot.
    /// If the binary data is larger than 49179 bytes, it is treated as a
    /// ZX Spectrum 128 snapshot, and the extension fields are populated.
    /// Otherwise, it is treated as a ZX Spectrum 48 snapshot.
    /// The memory is allocated to hold the snapshot data, and the relevant
    /// parts of the binary data are copied into the memory.
    /// The function returns a `Snapshot` instance with the initialized fields.
    /// ///
    /// # Arguments
    /// * `bin` - A byte slice containing the binary data of the snapshot.
    ///
    /// # Returns
    /// A `Snapshot` instance initialized with the data from the binary slice.
    fn try_from(bin: Vec<u8>) -> Result<Self, Self::Error> {
        const HEADER_SIZE: usize = std::mem::size_of::<SnapshotHeader>();
        let mut mapping: [u8; 3] = [0, 1, 2];  // assume 48k mapping (for now)

        let mut banks: Vec<Vec<u8>> = Vec::new();

        let mut extension = None;
        let mut snapshot_type = SnapshotType::Snapshot48;

        if bin.len() > MEM_48K + HEADER_SIZE {

            snapshot_type = SnapshotType::Snapshot128;
            extension = Some(SnapshotExtension {
                pc: u16::from_le_bytes([bin[49179], bin[49180]]),
                x7ffd: bin[49181],
                tr_dos: bin[49182],
            });

            // allocate 128K in 8 memory banks
            banks.push(vec![0u8; MEM_16K]); // bank 0
            banks.push(vec![0u8; MEM_16K]); // bank 1
            banks.push(vec![0u8; MEM_16K]); // bank 2
            banks.push(vec![0u8; MEM_16K]); // bank 3
            banks.push(vec![0u8; MEM_16K]); // bank 4
            banks.push(vec![0u8; MEM_16K]); // bank 5
            banks.push(vec![0u8; MEM_16K]); // bank 6
            banks.push(vec![0u8; MEM_16K]); // bank 7

            mapping[0] = 5; // bank 0
            mapping[1] = 2; // bank 1
            mapping[2] = extension.as_ref().unwrap().x7ffd & 0x07; // bank 2

            // take care of the banks mapped to the lower 48k
            banks[5][0..MEM_16K].copy_from_slice(&bin[HEADER_SIZE..HEADER_SIZE + MEM_16K]);
            banks[2][0..MEM_16K].copy_from_slice(&bin[HEADER_SIZE + MEM_16K..HEADER_SIZE + (2 * MEM_16K)]);
            banks[mapping[2] as usize][0..MEM_16K].copy_from_slice(&bin[HEADER_SIZE + (2 * MEM_16K)..HEADER_SIZE + (3 * MEM_16K)]);

            // fill the rest of the banks with the remaining data
            let mut potential_banks = vec![0, 1, 3, 4, 6, 7];
            potential_banks.retain(|&x| x != mapping[2] as usize);

            let mut index = HEADER_SIZE + MEM_48K + std::mem::size_of::<SnapshotExtension>();
            for bank in potential_banks {
                banks[bank][0..MEM_16K].copy_from_slice(&bin[index..index + MEM_16K]);
                index += MEM_16K;
            }
        }
        else{
            // allocate 48K in 3 memory banks
            banks.push(vec![0u8; MEM_16K]);
            banks.push(vec![0u8; MEM_16K]);
            banks.push(vec![0u8; MEM_16K]);

            banks[0][0..MEM_16K].copy_from_slice(&bin[HEADER_SIZE..HEADER_SIZE + MEM_16K]);
            banks[1][0..MEM_16K].copy_from_slice(&bin[HEADER_SIZE + MEM_16K..HEADER_SIZE + (2 * MEM_16K)]);
            banks[2][0..MEM_16K].copy_from_slice(&bin[HEADER_SIZE + (2 * MEM_16K)..HEADER_SIZE + (3 * MEM_16K)]);
        }

        Ok(Snapshot {
            header : SnapshotHeader {
                i: bin[0],
                hl_prime: u16::from_le_bytes([bin[1], bin[2]]),
                de_prime: u16::from_le_bytes([bin[3], bin[4]]),
                bc_prime: u16::from_le_bytes([bin[5], bin[6]]),
                af_prime: u16::from_le_bytes([bin[7], bin[8]]),
                hl: u16::from_le_bytes([bin[9], bin[10]]),
                de: u16::from_le_bytes([bin[11], bin[12]]),
                bc: u16::from_le_bytes([bin[13], bin[14]]),
                iy: u16::from_le_bytes([bin[15], bin[16]]),
                ix: u16::from_le_bytes([bin[17], bin[18]]),
                interrupt: bin[19],
                r: bin[20],
                af: u16::from_le_bytes([bin[21], bin[22]]),
                sp: u16::from_le_bytes([bin[23], bin[24]]),
                int_mode: bin[25],
                border_color: bin[26],
            },
            snapshot_type,
            extension,
            banks,
            mapping
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    // iterates through the checksums for each bank in a 48k snapshot
    // and compares it to the expected values.
    #[test]
    fn test_48k_file() {
        let expected: [u16; 3] = [59066, 0, 11458];  // assume 48k mapping (for now)
        let file = File::open("48k.sna").expect("Failed to open snapshot file");
        let snapshot = Snapshot::try_from(file).expect("Failed to parse snapshot");
        for bank in 0..3 {
            let checksum = snapshot.checksum(bank);
            assert_eq!(checksum, expected[bank], "Checksum for bank {} is incorrect expected {}, got {}", bank, expected[bank], checksum);
        }
    }

    // iterates through the checksums for each bank in a 128k snapshot
    // and compares it to the expected values.
    #[test]
    fn test_128k_file() {
        let expected: [u16; 8] = [12174, 0, 0, 0, 0, 46342, 0, 10827];
        let file = File::open("128k.sna").expect("Failed to open snapshot file");
        let snapshot = Snapshot::try_from(file).expect("Failed to parse snapshot");
        assert_eq!(snapshot.snapshot_type, SnapshotType::Snapshot128, "Snapshot type is not Snapshot128");
        for bank in 0..=7 {
            let checksum = snapshot.checksum(bank);
            assert_eq!(checksum, expected[bank], "Checksum for bank {} is incorrect expected {}, got {}", bank, expected[bank], checksum);
        }
    }

    /// Iterates throught the banks of a 128k snapshot, switches that bank into the 0xC000-0xCFFF memory range
    /// and checks that the checksum matches the expected value for that bank, and that the checksum matches the
    /// mapped memory checksum.
    #[test]
    fn test_port_7ffd() {
        let expected: [u16; 8] = [12174, 0, 0, 0, 0, 46342, 0, 10827];
        let file = File::open("128k.sna").expect("Failed to open snapshot file");
        let mut snapshot = Snapshot::try_from(file).expect("Failed to parse snapshot");
        assert_eq!(snapshot.snapshot_type, SnapshotType::Snapshot128, "Snapshot type is not Snapshot128");
        for bank in 0..=7 {
            snapshot.write_0x7ffd(bank as u8);
            let checksum = snapshot.checksum(bank);
            assert_eq!(checksum, expected[bank], "Checksum for bank {} is incorrect expected {}, got {}", bank, expected[bank], checksum);
            let mapped_checksum = {
                let mut sum: u16 = 0;
                for i in 0xC000..=0xFFFF {
                    sum = sum.wrapping_add(snapshot.peek(i) as u16);
                }
                sum
            };
            assert_eq!(checksum, mapped_checksum, "Checksum for bank {} is incorrect expected {}, got {}", bank, mapped_checksum, checksum);
        }
    }

    #[test]
    fn test_bank_peek() {
        let mut rng = rand::rng();
        let file = File::open("128k.sna").expect("Failed to open snapshot file");
        let mut snapshot = Snapshot::try_from(file).expect("Failed to parse snapshot");

        assert_eq!(snapshot.snapshot_type, SnapshotType::Snapshot128, "Snapshot type is not Snapshot128");
        for bank in 0..=7 {
            let mut mapped_checksum: u16 = 0;
            snapshot.write_0x7ffd(bank as u8);
            for i in 0xC000..=0xFFFF {
                let random_number: u8 = rng.random();
                snapshot.poke(i, random_number);
                mapped_checksum = mapped_checksum.wrapping_add(random_number as u16);
            }

            let mut bank_checksum: u16 = 0;
            for i in 0..=0x3FFF {
                let value = snapshot.bank_peek(bank, i);
                bank_checksum = bank_checksum.wrapping_add(value as u16);
            }

            assert_eq!(bank_checksum, mapped_checksum, "Banked checksum for bank {} is incorrect expected {}, got {}", bank, mapped_checksum, bank_checksum);
        }
    }

    #[test]
    fn test_bank_poke() {
        let mut rng = rand::rng();
        let file = File::open("128k.sna").expect("Failed to open snapshot file");
        let mut snapshot = Snapshot::try_from(file).expect("Failed to parse snapshot");

        assert_eq!(snapshot.snapshot_type, SnapshotType::Snapshot128, "Snapshot type is not Snapshot128");
        for bank in 0..=7 {
            let mut bank_checksum: u16 = 0;
            for i in 0..=0x3FFF {
                let random_number: u8 = rng.random();
                snapshot.bank_poke(bank, i, random_number);
                bank_checksum = bank_checksum.wrapping_add(random_number as u16);
            }

            let mut mapped_checksum: u16 = 0;
            snapshot.write_0x7ffd(bank as u8);
            for i in 0xC000..=0xFFFF {
                let value = snapshot.peek(i);
                mapped_checksum = mapped_checksum.wrapping_add(value as u16);
            }

            assert_eq!(bank_checksum, mapped_checksum, "Banked checksum for bank {} is incorrect expected {}, got {}", bank, mapped_checksum, bank_checksum);
        }
    }
}
