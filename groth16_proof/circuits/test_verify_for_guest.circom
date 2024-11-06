pragma circom 2.0.4;

include "../../circomlib/circuits/sha256/sha256.circom";
include "../../circomlib/circuits/bitify.circom";
include "risc0.circom";
include "blake3_compression.circom";
include "test_stark_verify.circom";

template Blake3 () {
    signal input inp[16]; // 16 32-bit words
    signal output out[8]; // 8 32-bit words

    component iv = IV();
    component blake3 = Blake3Compression();
    for (var i = 0; i < 8; i++) {
        blake3.h[i] <== iv.out[i];
    }
    for (var i = 0; i < 16; i++) {
        blake3.m[i] <== inp[i];
    }
    blake3.t[0] <== 0;
    blake3.t[1] <== 0;
    blake3.b <== 64;
    blake3.d <== 11;


    for (var i = 0; i < 8; i++) {
        out[i] <== blake3.out[i];
    }
}

template Blake3_with_scalar_output () {
    signal input inp[16]; // 16 32-bit words
    signal output out;
    signal outbits[256];

    component blake3 = Blake3();
    for (var i = 0; i < 16; i++) {
        blake3.inp[i] <== inp[i];
    }

    component to_bits[8];
    for (var i = 0; i < 8; i++) {
        to_bits[i] = to_bits_exact(32);
        to_bits[i].in <== blake3.out[i];
        log("blake3.out[i]");
        log(blake3.out[i]);
        for (var j = 0; j < 32; j++) {
            outbits[i*32 + j] <== to_bits[i].out[j];
        }
    }
    log("outbits");
    for (var i = 0; i < 256; i++) {
        log(outbits[i]);
    }

    component to_num = Bits2Num(248); // We delete the last 8 bits

    for (var i = 0; i < 248; i++) {
        var index = (248 - i - 1) - (248 - i - 1)%8 + i%8;
        to_num.in[index] <== outbits[i];
    }
    out <== to_num.out;
}

// Here, we take the journal (commitments of the stark_verify guest) and generate the claim_digest, which corresponds to the out[2], out[3] of the iop.
template Journal() {
    signal input journal_bits_in[256]; // journal in bits
    signal input pre_state_digest_bits[256]; // pre_state_digest in bits, this is kind of needed since it changes each time a different guest circuit is used. But it is constant for a given circuit.
    signal input post_state_digest_bits[256];
    signal output out[2]; // claim_digest in [u128, u128]


    // post_digest = 74032234001928697417723972706442396998535281042410084191249638976189322148066; // Constant no matter the circuit
    // var post_digest_bits[256] = [1,0,1,0,0,0,1,1,1,0,1,0,1,1,0,0,1,1,0,0,0,0,1,0,0,1,1,1,0,0,0,1,0,0,0,1,0,1,1,1,0,1,0,0,0,0,0,1,1,0,0,0,1,0,0,1,1,0,0,1,0,1,1,0,0,0,1,1,0,1,0,0,0,0,0,0,1,0,1,1,1,0,0,0,0,1,0,0,1,1,1,0,0,1,0,1,1,0,1,0,1,0,0,1,0,0,0,0,1,1,1,1,0,0,1,1,1,1,1,0,1,1,1,1,0,1,0,0,1,1,0,0,0,1,0,0,1,0,0,1,1,1,0,1,0,0,1,0,0,0,1,0,1,1,0,0,0,1,1,1,1,0,0,1,1,1,1,0,0,1,0,0,0,1,0,0,1,0,1,0,1,0,1,0,1,1,0,1,1,0,0,0,0,0,1,0,0,0,1,0,1,1,1,0,1,1,0,0,1,0,0,1,1,1,0,0,0,0,1,1,0,0,0,1,0,0,1,1,1,1,1,0,0,0,0,1,1,1,1,0,1,0,1,1,1,0,0,0,1,1,1,0,0,0,1,0];

    // CONSTANTS

    // output_tag = 54240429074780157751094335427642025272505087342533963631932091736940340402644; // Constant = hash("risc0.Output")
    var output_tag_bits[256] = [0,1,1,1,0,1,1,1,1,1,1,0,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,1,0,0,1,1,0,1,1,0,0,1,1,0,1,0,1,0,0,1,1,1,1,0,0,0,1,0,1,1,0,1,0,0,0,1,1,1,0,1,1,1,0,1,0,0,0,1,1,1,1,1,0,1,1,1,1,0,0,0,0,0,1,1,0,1,0,1,1,1,1,0,1,1,1,0,1,1,0,0,0,1,0,1,1,1,0,1,1,0,0,0,1,0,1,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,1,0,1,1,1,1,1,1,1,1,1,0,1,0,1,0,1,0,1,0,1,1,0,0,1,0,0,1,0,0,0,1,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,1,0,0,1,1,0,1,0,0,1,0,1,1,0,1,1,1,1,1,0,0,1,1,0,0,0,1,1,1,1,0,1,1,0,1,0,0,0,1,1,0,0,1,0,1,1,0,1,0,0,1,1,0,1,0,1,0,1,0,1,1,0,0,1,1,1,0,1,0,1,0,0];
    
    // TODO: Add comment here
    var receipt_claim_tag_bits[256] = [1,1,0,0,1,0,1,1,0,0,0,1,1,1,1,1,1,1,1,0,1,1,1,1,1,1,0,0,1,1,0,1,0,0,0,1,1,1,1,1,0,0,1,0,1,1,0,1,1,0,0,1,1,0,1,0,0,1,1,0,0,1,0,0,1,0,0,1,0,1,1,1,0,1,0,1,1,1,0,0,1,0,1,1,1,0,1,1,1,0,1,1,1,1,1,1,0,1,1,0,1,1,1,0,0,0,0,1,0,1,1,0,0,0,0,1,1,1,1,0,0,0,1,0,1,0,0,1,0,0,0,1,0,1,0,0,0,1,0,0,0,0,1,1,0,1,0,0,1,0,1,1,0,0,0,0,1,1,0,0,1,0,1,1,1,0,1,1,1,0,0,1,1,0,0,1,0,1,1,0,0,0,0,0,1,0,1,1,1,0,0,0,0,1,0,0,1,1,0,1,1,1,1,1,0,1,0,1,1,1,0,1,0,1,1,1,0,0,0,1,0,1,1,1,1,1,1,0,1,0,0,0,0,1,1,0,1,0,1,1,0,1,0,0,1,0,0,0,1,0,1,0,1,1,1,1];

    var bits_0200[16] = [0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0];
    var bits_0400[16] = [0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0];


    component journal_hasher = Sha256(256); // Depends on journal, hash(journal)
    journal_hasher.in <== journal_bits_in;


    component output_hasher = Sha256(784); // Depends on journal, hash(output_tag_hash, journal_digest, [0u8; 32], 0200)
    for (var i = 0; i < 256; i++) {
        output_hasher.in[i] <== output_tag_bits[i];
    }
    for(var i = 0; i < 256; i++) {
        output_hasher.in[256 + i] <== journal_hasher.out[i];
    }
    for (var i = 0; i < 256; i++) {
        output_hasher.in[512 + i] <== 0;
    }
    for(var i = 0; i < 16; i++){
        output_hasher.in[768 + i] <== bits_0200[i];
    }


    component claim_hasher = Sha256(1360); // hash(receipt_claim_tag_hash, claim_input_digest, claim_pre_digest, claim_post_digest, claim_output_digest, 0000000000000000 0400)
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
        claim_hasher.in[768 + i] <== post_state_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        claim_hasher.in[1024 + i] <== output_hasher.out[i];
    }
    for (var i = 0; i < 64; i++) {
        claim_hasher.in[1280 + i] <== 0;
    }
    for (var i = 0; i < 16; i++) {
        claim_hasher.in[1344 + i] <== bits_0400[i];
    }

    component b2n_1 = Bits2Num(128);
    for(var i = 0; i < 16; i++) {
        for(var j = 0; j < 8; j++) {
            b2n_1.in[8 * i + 7 - j] <== claim_hasher.out[8 * i + j];
        }
    }

    component b2n_2 = Bits2Num(128);
    for(var i = 0; i < 16; i++) {
        for(var j = 0; j < 8; j++) {
            b2n_2.in[8 * i + 7 - j] <== claim_hasher.out[128 + 8 * i + j];
        }
    }

    out[0] <== b2n_1.out;
    out[1] <== b2n_2.out;
}

template VerifyForGuest() {
    signal input iop[25749]; // Succinct proof from the STARK circuit.
    signal input journal_digest_bits[256]; // We assume the journal is 32 bytes long, so 256 bits.
    signal input control_root[2]; // This is the control root of the STARK circuit, sort of a Merkle root of some stuff I do not know by heart. CONSTANT FOR A GIVEN CIRCUIT.
    signal input pre_state_digest_bits[256]; // This is the pre-state digest of the STARK circuit. CONSTANT FOR A GIVEN CIRCUIT.
    signal input post_state_digest_bits[256]; // This is the post-state digest of the STARK circuit. CONSTANT FOR A GIVEN CIRCUIT.
    signal input id_bn254_fr_bits[256]; // This is the code root of the STARK circuit. CONSTANT FOR A GIVEN CIRCUIT.
    signal output final_blake3_digest; // This will be Blake3(ALL_CONSTANTS, journal_blake3_digest) and its first 248 bits.


    // VERIFY STARK CIRCUIT

    component stark_verifier = Verify();
    for (var i = 0; i < 25749; i++) {
        stark_verifier.iop[i] <== iop[i];
    }

    component claim = Journal();
    for (var i = 0; i < 256; i++) {
        claim.journal_bits_in[i] <== journal_digest_bits[i];
    }

    log("journal_digest_bits");
    for (var i = 0; i < 256; i++) {
        log(journal_digest_bits[i]);
    }

    for (var i = 0; i < 256; i++) {
        claim.pre_state_digest_bits[i] <== pre_state_digest_bits[i];
    }

    log("pre_state_digest_bits");
    for (var i = 0; i < 256; i++) {
        log(pre_state_digest_bits[i]);
    }

    for (var i = 0; i < 256; i++) {
        claim.post_state_digest_bits[i] <== post_state_digest_bits[i];
    }

    log("post_state_digest_bits");
    for (var i = 0; i < 256; i++) {
        log(post_state_digest_bits[i]);
    }


    // This gives the code root of the STARK circuit. Bytes of id_bn254_fr_bits reversed.
    component id_bn254_fr_b2n = Bits2Num(256);
    for (var i = 0; i < 32; i++) {
        for (var j = 0; j < 8; j++) {
            id_bn254_fr_b2n.in[255 - (8 * i + j)] <== id_bn254_fr_bits[8 * (31 - i) + j];
        }
    }


    stark_verifier.out[0] === control_root[0];
    stark_verifier.out[1] === control_root[1];
    stark_verifier.out[2] === claim.out[0];
    stark_verifier.out[3] === claim.out[1];
    stark_verifier.codeRoot === id_bn254_fr_b2n.out;

    log("control_root[0]", control_root[0]);
    log("control_root[1]", control_root[1]);
    log("claim.out[0]", claim.out[0]);
    log("claim.out[1]", claim.out[1]);
    log("id_bn254_fr_b2n.out", id_bn254_fr_b2n.out);

    // PREPARE FINAL BLAKE3 DIGEST

    component control_root_n2b[2];
    for (var i = 0; i < 2; i++) {
        control_root_n2b[i] = Num2Bits(128);
        control_root_n2b[i].in <== control_root[i];
    }
    
    component constants_hasher = Sha256(1024);
    for (var i = 0; i < 2; i++) {
        for (var j = 0; j < 128; j++) {
            constants_hasher.in[i * 128 + j] <== control_root_n2b[i].out[j];
        }
    }
    log("control_root_n2b[0]");
    for (var i = 0; i < 128; i++) {
        log(control_root_n2b[0].out[i]);
    }
    log("control_root_n2b[1]");
        for (var i = 0; i < 128; i++) {
        log(control_root_n2b[1].out[i]);
    }
    for (var i = 0; i < 256; i++) {
        constants_hasher.in[256 + i] <== pre_state_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        constants_hasher.in[512 + i] <== post_state_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        constants_hasher.in[768 + i] <== id_bn254_fr_bits[i];
    }

    log("constants digest");
    for (var i = 0; i < 256; i++) {
        log(constants_hasher.out[i]);
    }

    log("journal_digest");
    for (var i = 0; i < 256; i++) {
        log(journal_digest_bits[i]);
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
            bits_to_u32[8 + i].in[j] <== journal_digest_bits[i * 32 + j];
        }
    }

    log("bits_to_u32");
    for (var i = 0; i < 16; i++) {
        log(bits_to_u32[i].out);
    }

    component final_hasher = Blake3_with_scalar_output();

    for (var i = 0; i < 16; i++) {
        final_hasher.inp[i] <== bits_to_u32[i].out;
    }
    final_blake3_digest <== final_hasher.out;

    log("final_blake3_digest", final_blake3_digest);

    // component bits_to_u32[16];
    // for (var i = 1; i < 16; i++) {
    //     bits_to_u32[i] = Bits2Num(32);
    //     for (var j = 0; j < 32; j++) {
    //         bits_to_u32[i].in[j] <== 0;
    //     }
    // }
    // bits_to_u32[0] = Bits2Num(32);
    // bits_to_u32[15] = Bits2Num(32);
    // for (var j = 1; j < 32; j++) {
    //     bits_to_u32[0].in[j] <== 0;
    // }
    // bits_to_u32[0].in[0] <== 1;
    // for (var j = 0; j < 30; j++) {
    //     bits_to_u32[15].in[j] <== 0;
    // }
    // bits_to_u32[15].in[30] <== 1;
    // bits_to_u32[15].in[31] <== 0;

    // log("bits_to_u32");
    // for (var i = 0; i < 16; i++) {
    //     log(bits_to_u32[i].out);
    // }

    //  component final_hasher = Blake3_with_scalar_output();

    // for (var i = 0; i < 16; i++) {
    //     final_hasher.inp[i] <== bits_to_u32[i].out;
    // }
    // final_blake3_digest <== final_hasher.out;

    // log("final_blake3_digest", final_blake3_digest);

}

component main = VerifyForGuest();