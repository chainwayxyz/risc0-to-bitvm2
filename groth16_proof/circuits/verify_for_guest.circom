pragma circom 2.0.4;

include "../../circomlib/circuits/sha256/sha256.circom";
include "../../circomlib/circuits/bitify.circom";
include "risc0.circom";

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
    signal input journal_digest_bits[256]; // We assume the journal is 32 bytes long, so 256 bits.
    signal input control_root[2]; // This is the control root of the STARK circuit, sort of a Merkle root of some stuff I do not know by heart. CONSTANT FOR A GIVEN CIRCUIT.
    signal input pre_state_digest_bits[256]; // This is the pre-state digest of the STARK circuit. CONSTANT FOR A GIVEN CIRCUIT.
    signal input post_state_digest_bits[256]; // This is the post-state digest of the STARK circuit. CONSTANT FOR A GIVEN CIRCUIT.
    signal input id_bn254_fr_bits[254]; // This is the code root of the STARK circuit. CONSTANT FOR A GIVEN CIRCUIT.
    signal output out[5]; // out[0] = control_root[0], out[1] = control_root[1], out[2] = claim.out[0], out[3] = claim.out[1], out[4] = id_bn254_fr_b2n.out


    // BINARY CHECKS
    for (var i = 0; i < 256; i++) {
        journal_digest_bits[i] * (1 - journal_digest_bits[i]) === 0;
    }
    for (var i = 0; i < 256; i++) {
        pre_state_digest_bits[i] * (1 - pre_state_digest_bits[i]) === 0;
    }
    for (var i = 0; i < 256; i++) {
        post_state_digest_bits[i] * (1 - post_state_digest_bits[i]) === 0;
    }
    for (var i = 0; i < 254; i++) {
        id_bn254_fr_bits[i] * (1 - id_bn254_fr_bits[i]) === 0;
    }


    // VERIFY STARK CIRCUIT

    component claim = Journal();
    for (var i = 0; i < 256; i++) {
        claim.journal_bits_in[i] <== journal_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        claim.pre_state_digest_bits[i] <== pre_state_digest_bits[i];
    }
    for (var i = 0; i < 256; i++) {
        claim.post_state_digest_bits[i] <== post_state_digest_bits[i];
    }

    // This gives the code root of the STARK circuit. Bytes of id_bn254_fr_bits reversed.
    component id_bn254_fr_b2n = Bits2Num_strict();
    for (var i = 0; i < 31; i++) {
        for (var j = 0; j < 8; j++) {
            id_bn254_fr_b2n.in[255 - 8 * (i + 1) - j] <== id_bn254_fr_bits[248 - 8 * (i + 1) + j];
        }
    }

    for (var j = 0; j < 6; j++) {
        id_bn254_fr_b2n.in[253 - j] <== id_bn254_fr_bits[248 + j];
    }

    // OUTPUTS
    out[0] <== control_root[0];
    out[1] <== control_root[1];
    out[2] <== claim.out[0];
    out[3] <== claim.out[1];
    out[4] <== id_bn254_fr_b2n.out;

}

component main = VerifyForGuest();
