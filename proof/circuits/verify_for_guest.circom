pragma circom 2.0.4;

include "stark_verify.circom";
include "blake3.circom";
include "../../circomlib/circuits/sha256/sha256.circom";
include "../../circomlib/circuits/bitify.circom";

// Here, we take the journal (commitments of the stark_verify guest) and generate the claim_digest, which corresponds to the out[2], out[3] of the iop.
template Journal() {
    signal input journal_bits_in[256]; // journal in bits
    signal input pre_state_digest_bits[256]; // pre_state_digest in bits, this is kind of needed since it changes each time a different guest circuit is used. But it is constant for a given circuit.
    signal output out[2]; // claim_digest in [u128, u128]
    component claim_hasher = Sha256(1360); // hash(receipt_claim_tag_hash, claim_input_digest, claim_pre_digest, claim_post_digest, claim_output_digest, 00000000000000000400)
    component output_hasher = Sha256(784); // Depends on journal, hash(output_tag_hash, journal_digest, [0u8; 32], 0200)
    // post_digest = 74032234001928697417723972706442396998535281042410084191249638976189322148066; // Constant no matter the circuit
    var post_digest_bits[256] = [1,0,1,0,0,0,1,1,1,0,1,0,1,1,0,0,1,1,0,0,0,0,1,0,0,1,1,1,0,0,0,1,0,0,0,1,0,1,1,1,0,1,0,0,0,0,0,1,1,0,0,0,1,0,0,1,1,0,0,1,0,1,1,0,0,0,1,1,0,1,0,0,0,0,0,0,1,0,1,1,1,0,0,0,0,1,0,0,1,1,1,0,0,1,0,1,1,0,1,0,1,0,0,1,0,0,0,0,1,1,1,1,0,0,1,1,1,1,1,0,1,1,1,1,0,1,0,0,1,1,0,0,0,1,0,0,1,0,0,1,1,1,0,1,0,0,1,0,0,0,1,0,1,1,0,0,0,1,1,1,1,0,0,1,1,1,1,0,0,1,0,0,0,1,0,0,1,0,1,0,1,0,1,0,1,1,0,1,1,0,0,0,0,0,1,0,0,0,1,0,1,1,1,0,1,1,0,0,1,0,0,1,1,1,0,0,0,0,1,1,0,0,0,1,0,0,1,1,1,1,1,0,0,0,0,1,1,1,1,0,1,0,1,1,1,0,0,0,1,1,1,0,0,0,1,0];
    var output_tag_bits[256] = [0,1,1,1,0,1,1,1,1,1,1,0,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,1,0,0,1,1,0,1,1,0,0,1,1,0,1,0,1,0,0,1,1,1,1,0,0,0,1,0,1,1,0,1,0,0,0,1,1,1,0,1,1,1,0,1,0,0,0,1,1,1,1,1,0,1,1,1,1,0,0,0,0,0,1,1,0,1,0,1,1,1,1,0,1,1,1,0,1,1,0,0,0,1,0,1,1,1,0,1,1,0,0,0,1,0,1,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,1,0,1,1,1,1,1,1,1,1,1,0,1,0,1,0,1,0,1,0,1,1,0,0,1,0,0,1,0,0,0,1,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,1,0,0,1,1,0,1,0,0,1,0,1,1,0,1,1,1,1,1,0,0,1,1,0,0,0,1,1,1,1,0,1,1,0,1,0,0,0,1,1,0,0,1,0,1,1,0,1,0,0,1,1,0,1,0,1,0,1,0,1,1,0,0,1,1,1,0,1,0,1,0,0];
    // output_tag = 54240429074780157751094335427642025272505087342533963631932091736940340402644; // Constant = hash("risc0.Output")

    component journal_hasher = Sha256(256); // Depends on journal, hash(journal)

    // log("journal_bits");
    // for (var i = 0; i < 256; i++) {
    //     log(journal_bits_in[i]);
    // }
    
    journal_hasher.in <== journal_bits_in;

    // log("journal_digest");
    // for (var i = 0; i < 256; i++) {
    //     log(journal_hasher.out[i]);
    // }
    
    for (var i = 0; i < 256; i++) {
        output_hasher.in[i] <== output_tag_bits[i];
    }
    for(var i = 0; i < 256; i++) {
        output_hasher.in[256 + i] <== journal_hasher.out[i];
    }
    for (var i = 0; i < 256; i++) {
        output_hasher.in[512 + i] <== 0;
    }

    output_hasher.in[768] <== 0;
    output_hasher.in[769] <== 0;
    output_hasher.in[770] <== 0;
    output_hasher.in[771] <== 0;
    output_hasher.in[772] <== 0;
    output_hasher.in[773] <== 0;
    output_hasher.in[774] <== 1;
    output_hasher.in[775] <== 0;
    output_hasher.in[776] <== 0;
    output_hasher.in[777] <== 0;
    output_hasher.in[778] <== 0;
    output_hasher.in[779] <== 0;
    output_hasher.in[780] <== 0;
    output_hasher.in[781] <== 0;
    output_hasher.in[782] <== 0;
    output_hasher.in[783] <== 0;

    var receipt_claim_tag_bits[256] = [1,1,0,0,1,0,1,1,0,0,0,1,1,1,1,1,1,1,1,0,1,1,1,1,1,1,0,0,1,1,0,1,0,0,0,1,1,1,1,1,0,0,1,0,1,1,0,1,1,0,0,1,1,0,1,0,0,1,1,0,0,1,0,0,1,0,0,1,0,1,1,1,0,1,0,1,1,1,0,0,1,0,1,1,1,0,1,1,1,0,1,1,1,1,1,1,0,1,1,0,1,1,1,0,0,0,0,1,0,1,1,0,0,0,0,1,1,1,1,0,0,0,1,0,1,0,0,1,0,0,0,1,0,1,0,0,0,1,0,0,0,0,1,1,0,1,0,0,1,0,1,1,0,0,0,0,1,1,0,0,1,0,1,1,1,0,1,1,1,0,0,1,1,0,0,1,0,1,1,0,0,0,0,0,1,0,1,1,1,0,0,0,0,1,0,0,1,1,0,1,1,1,1,1,0,1,0,1,1,1,0,1,0,1,1,1,0,0,0,1,0,1,1,1,1,1,1,0,1,0,0,0,0,1,1,0,1,0,1,1,0,1,0,0,1,0,0,0,1,0,1,0,1,1,1,1];

    for (var i = 0; i < 256; i++) {
        claim_hasher.in[i] <== receipt_claim_tag_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        claim_hasher.in[256 + i] <== 0; // Input digest, constant no matter the circuit
    }
    for (var i = 0; i < 256; i++) {
        claim_hasher.in[512 + i] <== pre_state_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        claim_hasher.in[768 + i] <== post_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        claim_hasher.in[1024 + i] <== output_hasher.out[i];
    }
    for (var i = 0; i < 64; i++) {
        claim_hasher.in[1280 + i] <== 0;
    }

    claim_hasher.in[1344] <== 0;
    claim_hasher.in[1345] <== 0;
    claim_hasher.in[1346] <== 0;
    claim_hasher.in[1347] <== 0;
    claim_hasher.in[1348] <== 0;
    claim_hasher.in[1349] <== 1;
    claim_hasher.in[1350] <== 0;
    claim_hasher.in[1351] <== 0;
    claim_hasher.in[1352] <== 0;
    claim_hasher.in[1353] <== 0;
    claim_hasher.in[1354] <== 0;
    claim_hasher.in[1355] <== 0;
    claim_hasher.in[1356] <== 0;
    claim_hasher.in[1357] <== 0;
    claim_hasher.in[1358] <== 0;
    claim_hasher.in[1359] <== 0;

    // log("sha digest");
    // for (var i = 0; i < 256; i++) {
    //     log(claim_hasher.out[i]);
    // }

    component b2n_1 = Bits2Num(128);
    for(var i = 0; i < 16; i++) {
        for(var j = 0; j < 8; j++) {
            b2n_1.in[8 * i + 7 - j] <== claim_hasher.out[8 * i + j];
        }
    }
    out[0] <== b2n_1.out;

    component b2n_2 = Bits2Num(128);
    for(var i = 0; i < 16; i++) {
        for(var j = 0; j < 8; j++) {
            b2n_2.in[8 * i + 7 - j] <== claim_hasher.out[128 + 8 * i + j];
        }
    }
    out[1] <== b2n_2.out;
}

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
