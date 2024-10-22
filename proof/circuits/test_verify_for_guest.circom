pragma circom 2.0.4;

include "test_stark_verify.circom";
include "test_journal.circom";
include "blake3.circom";

template VerifyForGuest() {
    signal input journal_blake3_digest_bits[256]; // This comes from the verify_stark guest. A constant-sized journal.
    signal input control_root[2]; // This is the control root of the STARK circuit, sort of a Merkle root of some stuff I do not know by heart. CONSTANT FOR A GIVEN CIRCUIT.
    signal input pre_state_digest_bits[256]; // This is the pre-state digest of the STARK circuit. CONSTANT FOR A GIVEN CIRCUIT.
    signal input id_bn254_fr_bits[256]; // This is the code root of the STARK circuit. CONSTANT FOR A GIVEN CIRCUIT.
    signal output final_blake3_digest; // This will be Blake3(ALL_CONSTANTS, journal_blake3_digest) and its first 248 bits.

    component control_root_n2b[2];
    for (var i = 0; i < 2; i++) {
        control_root_n2b[i] = Num2Bits(128);
        control_root_n2b[i].in <== control_root[i];
    }

    component constants_hasher = Sha256(768);
    for (var i = 0; i < 2; i++) {
        for (var j = 0; j < 128; j++) {
            constants_hasher.in[i * 128 + j] <== control_root_n2b[i].out[j];
        }
    }
    for (var i = 0; i < 256; i++) {
        constants_hasher.in[256 + i] <== pre_state_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        constants_hasher.in[512 + i] <== id_bn254_fr_bits[i];
    }

    component bits_to_u32[16];
    for (var i = 0; i < 8; i++) {
        bits_to_u32[i] = Bits2Num(32);
        for (var j = 0; j < 32; j++) {
            bits_to_u32[i].in[j] <== constants_hasher.out[i * 32 + j];
        }
    }
    for (var i = 0; i < 8; i++) {
        bits_to_u32[i + 8] = Bits2Num(32);
        for (var j = 0; j < 32; j++) {
            bits_to_u32[8 + i].in[j] <== journal_blake3_digest_bits[i * 32 + j];
        }
    }

    // component pre_state_digest_bits_b2n = Bits2Num(256);
    // for (var i = 0; i < 256; i++) {
    //     pre_state_digest_bits_b2n.in[i] <== pre_state_digest_bits[i];
    // }

    // This gives the code root of the STARK circuit. Bytes of id_bn254_fr_bits reversed.
    component id_bn254_fr_b2n = Bits2Num(256);
    for (var i = 0; i < 32; i++) {
        for (var j = 0; j < 8; j++) {
            id_bn254_fr_b2n.in[255 - (8 * i + j)] <== id_bn254_fr_bits[8 * (31 - i) + j];
        }
    }

    signal input iop[25749];
    component stark_verifier = Verify();
    component claim = Journal();

    for (var i = 0; i < 25749; i++) {
        stark_verifier.iop[i] <== iop[i];
    }

    for (var i = 0; i < 256; i++) {
        claim.journal_bits_in[i] <== journal_blake3_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        claim.pre_state_digest_bits[i] <== pre_state_digest_bits[i];
    }

    // log("out_0");
    // log(stark_verifier.out[0]);
    // log("out_1");
    // log(stark_verifier.out[1]);
    // log("out_2");
    // log(stark_verifier.out[2]);
    // log("out_3");
    // log(stark_verifier.out[3]);
    // log("codeRoot");
    // log(stark_verifier.codeRoot);
    // log("claim_out_0");
    // log(claim.out[0]);
    // log("claim_out_1");
    // log(claim.out[1]);
    // log("id_bn254_fr_b2n.out");
    // log(id_bn254_fr_b2n.out);

    stark_verifier.out[0] === control_root[0];
    stark_verifier.out[1] === control_root[1]; 
    stark_verifier.out[2] === claim.out[0];
    stark_verifier.out[3] === claim.out[1];

    component final_hasher = Blake3_with_scalar_output();

    for (var i = 0; i < 16; i++) {
        final_hasher.inp[i] <== bits_to_u32[i].out;
    }
    final_blake3_digest <== final_hasher.out;

    stark_verifier.codeRoot === id_bn254_fr_b2n.out;
}

component main = VerifyForGuest();
