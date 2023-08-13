// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

impl<E: Environment> ToBits for Group<E> {
    /// Outputs the little-endian bit representation of `self.to_x_coordinate()` *without* trailing zeros.
    fn write_bits_le(&self, vec: &mut Vec<bool>) {
        self.to_x_coordinate().write_bits_le(vec);
    }

    /// Outputs the big-endian bit representation of `self.to_x_coordinate()` *without* leading zeros.
    fn write_bits_be(&self, vec: &mut Vec<bool>) {
        self.to_x_coordinate().write_bits_be(vec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network_environment::Console;

    type CurrentEnvironment = Console;

    const ITERATIONS: u64 = 10_000;

    #[test]
    fn test_to_bits_le() {
        let mut rng = TestRng::default();

        for _ in 0..ITERATIONS {
            // Sample a random value.
            let group: Group<CurrentEnvironment> = Uniform::rand(&mut rng);

            let candidate = group.to_bits_le();
            assert_eq!(Group::<CurrentEnvironment>::size_in_bits(), candidate.len());

            for (expected, candidate) in (*group).to_affine().to_x_coordinate().to_bits_le().iter().zip_eq(&candidate) {
                assert_eq!(expected, candidate);
            }
        }
    }

    #[test]
    fn test_to_bits_be() {
        let mut rng = TestRng::default();

        for _ in 0..ITERATIONS {
            // Sample a random value.
            let group: Group<CurrentEnvironment> = Uniform::rand(&mut rng);

            let candidate = group.to_bits_be();
            assert_eq!(Group::<CurrentEnvironment>::size_in_bits(), candidate.len());

            for (expected, candidate) in (*group).to_affine().to_x_coordinate().to_bits_be().iter().zip_eq(&candidate) {
                assert_eq!(expected, candidate);
            }
        }
    }
}
